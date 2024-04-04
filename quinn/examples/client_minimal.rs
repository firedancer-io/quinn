//! Minimal client for testing quinn <-> fd_quic compatibility.

mod common;
use std::{
    fs::File,
    io::BufReader,
};

use proto::ClientConfig;
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

    // let cert = if let Item::X509Certificate(cert) =
    //     rustls_pemfile::read_one(&mut BufReader::new(File::open("../firedancer/cert.pem").unwrap()))
    //         .unwrap()
    //         .unwrap()
    // {
    //     Some(cert)
    // } else {
    //     None
    // };
    // let cert = cert.unwrap();
    // let endpoint = make_client_endpoint("0.0.0.0:0".parse().unwrap(), &[cert.as_slice()]).unwrap();
    let endpoint = make_client_endpoint("0.0.0.0:0".parse().unwrap(), &[]).unwrap();
    let connection = endpoint
        .connect("127.0.0.1:1033".parse().unwrap(), "localhost")
        .unwrap()
        .await
        .unwrap();
    println!("[client] connected: addr={}", connection.remote_address());
    let _ = connection.accept_uni().await?;
    endpoint.wait_idle().await;
    Ok(())
}
