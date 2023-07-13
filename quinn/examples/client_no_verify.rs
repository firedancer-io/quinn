//! Minimal client for testing quinn <-> fd_quic compatibility.

mod common;
use std::{
    sync::Arc,
};

use proto::ClientConfig;
use rustls_pemfile::Item;

use crate::common::make_client_endpoint;

// Implementation of `ServerCertVerifier` that verifies everything as trustworthy.
struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();

    let crypto = rustls::ClientConfig::builder()
    .with_safe_defaults()
    .with_custom_certificate_verifier(SkipServerVerification::new())
    .with_no_client_auth();

    let mut client_config = ClientConfig::new(Arc::new(crypto));


    let mut endpoint = make_client_endpoint("0.0.0.0:0".parse().unwrap(), &[]).unwrap();
    endpoint.set_default_client_config(client_config);
    let connection = endpoint
        .connect("127.0.0.1:9001".parse().unwrap(), "localhost")
        .unwrap()
        .await
        .unwrap();
    println!("[client] connected: addr={}", connection.remote_address());
    let _ = connection.accept_uni().await?;
    endpoint.wait_idle().await;
    Ok(())
}
