//! openssl genpkey -algorithm ed25519 -out private.pem

mod common;
use std::{
    error::Error,
    fs::File,
    io::{self, BufReader, Write},
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
    time::{Instant, Duration},
};
use anyhow::{anyhow, Result};
use quinn::Endpoint;
use rcgen::{CertificateParams, DistinguishedName, DnType, SanType};
use rustls::PrivateKey;
use rustls_pemfile::Item;

fn load_private_key_from_file(path: &str) -> Result<PrivateKey, Box<dyn std::error::Error>> {
    let file = File::open(&path)?;
    let mut reader = BufReader::new(file);
    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut reader)?;
    match keys.len() {
        0 => Err(format!("No PKCS8-encoded private key found in {path}").into()),
        1 => Ok(PrivateKey(keys.remove(0))),
        _ => Err(format!("More than one PKCS8-encoded private key found in {path}").into()),
    }
}

pub fn new_self_signed_tls_certificate_chain(
    san: IpAddr,
) -> Result<(Vec<rustls::Certificate>, rustls::PrivateKey), Box<dyn Error>> {
    let mut cert_params = CertificateParams::default();
    cert_params.subject_alt_names = vec![SanType::IpAddress(san)];
    cert_params.alg = &rcgen::PKCS_ED25519;
    let pk = load_private_key_from_file("private.pem")?;
    cert_params.key_pair = Some(rcgen::KeyPair::from_der(&pk.0)?);
    cert_params.distinguished_name = DistinguishedName::new();
    cert_params
        .distinguished_name
        .push(DnType::CommonName, "Solana node");
    let cert = rcgen::Certificate::from_params(cert_params)?;
    let cert_der = cert.serialize_der().unwrap();
    let cert_chain = vec![rustls::Certificate(cert_der)];
    Ok((cert_chain, pk))
}

fn duration_secs(x: &Duration) -> f32 {
    x.as_secs() as f32 + x.subsec_nanos() as f32 * 1e-9
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )?;

    let cert = if let Item::X509Certificate(cert) =
        rustls_pemfile::read_one(&mut BufReader::new(File::open("../firedancer/cert.pem")?))
            .unwrap()
            .unwrap()
    {
        Some(cert)
    } else {
        None
    };
    let cert = cert.unwrap();

    let mut certs = rustls::RootCertStore::empty();
    certs.add(&rustls::Certificate(cert.to_vec()))?;

    let (cert_chain, key_der) =
        new_self_signed_tls_certificate_chain(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))?;

    let mut client_crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(certs)
        .with_client_auth_cert(cert_chain, key_der)?;
    client_crypto.alpn_protocols = vec![b"solana-tpu".to_vec()];

    let client_config = quinn::ClientConfig::new(Arc::new(client_crypto));

    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);

    let conn = endpoint
        .connect("127.0.0.1:9001".parse().unwrap(), "localhost")
        .unwrap()
        .await
        .unwrap();
    let start = Instant::now();
    eprintln!("connected at {:?}", start.elapsed());
    let (mut send, mut recv) = conn.open_bi().await?;

    send.write_all("hello world".as_bytes())
        .await
        .map_err(|e| anyhow!("failed to send request: {}", e))?;
    send.finish()
        .await
        .map_err(|e| anyhow!("failed to shutdown stream: {}", e))?;
    let response_start = Instant::now();
    eprintln!("request sent at {:?}", response_start - start);
    let resp = recv
        .read_to_end(usize::max_value())
        .await
        .map_err(|e| anyhow!("failed to read response: {}", e))?;
    let duration = response_start.elapsed();
    eprintln!(
        "response received in {:?} - {} KiB/s",
        duration,
        resp.len() as f32 / (duration_secs(&duration) * 1024.0)
    );
    io::stdout().write_all(&resp).unwrap();
    io::stdout().flush().unwrap();
    conn.close(0u32.into(), b"done");

    // Give the server a fair chance to receive the close packet
    endpoint.wait_idle().await;
    Ok(())
}
