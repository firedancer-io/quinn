//! Minimal client for testing quinn <-> fd_quic compatibility.

mod common;
use std::{
    fs::{self, File},
    io::BufReader,
    sync::Arc,
};

use rustls_pemfile::Item;

use crate::common::make_client_endpoint;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();

    // let certs = rustls_pemfile::certs(&mut BufReader::new(File::open("./cert.pem")?))?;
    let f = File::open("./cert.pem").unwrap();
    let cert = if let Item::X509Certificate(cert) = rustls_pemfile::read_one(&mut BufReader::new(f))
        .unwrap()
        .unwrap()
    {
        Some(cert)
    } else {
        None
    };
    let cert = cert.unwrap();
    let endpoint = make_client_endpoint("0.0.0.0:0".parse().unwrap(), &[cert.as_slice()]).unwrap();
    let connection = endpoint
        .connect("127.0.0.1:9000".parse().unwrap(), "localhost")
        .unwrap()
        .await
        .unwrap();
    println!("[client] connected: addr={}", connection.remote_address());
    let _ = connection.accept_uni().await?;
    endpoint.wait_idle().await;
    Ok(())
}
