//! Minimal client for testing quinn <-> fd_quic compatibility.

mod common;
use base64;
use std::{
    fs::File,
    io::BufReader, sync::Arc,
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
        .connect("198.18.0.1:9001".parse().unwrap(), "localhost")
        .unwrap()
        .await
        .unwrap();
    println!("[client] connected: addr={}", connection.remote_address());

    let mut buffer: [u8; 1200] = [0; 1200];
    let base64_string = "AbxB0Ant74jDC3ce6kZUsZWzBSQGkAlUAGDqL+WmYCYvlhDTvkA70KVOzVfOCKn6M3eM6Nrc+op5ZL2/85RpvAUBAA4fer1aX2iKqfYbp8+8oxZVblqFx24zLh64xZ9LVUMZPbYWVGRh/AvIpBkfn/MYPUdfpML0cODhEoDLdD+97dZ/IRzXHQoIqQD2IiepGlWrW1jjvatQ2qYan2v58LcGoYQSHXq+dAWzftqHaeGr0N1lzdOuE1LloFXGQPqe5lk4qaUkh7L1laTv0i1SD0yGYSrG3eyNyzvlXVoHOP5vFXPUfSxqbyrJUnpwtoUNZQmICjyBHyxWCtvdHGmSFbUiQ+RaQWJiWlQBQskECDxQ9OATYbErzTEArj+gKfEESKreXRZKHGcrJetgJzTebWR26mMH5zPVlCVHsBTFzVAj+Zsw42cimDSkgOQfOYPjSx3b3KJ6408bu+hUPohysTL1xSJOjqvgjWcRP0sv724wmdU14gQcPy6oDm5M3Q/M8aUyWpqa5285/oJYq/96i4xrMv+GWncSVqIkJ57a5T97UmF6zaYd9yczYuaMaDVHM/hoIMa+YHF0K4h5nyrDOAsq5V22vpp8OaJHE0O/sKjfMYX+t5gentZhj9Bn/xY9/EGnT2fDoYb28n5Iq6fX5n2VC5se1zE9zN1+l8eGHDjJb6iaHdDf6kdRWNiOqZQTomji6US9/HZnQqnKD6RQVOapwry53A0u1GXiTaJd1g+bICf5xaMfAnwNyXeAutsdGj1Rb1YIJy+6zgmB+cRTPAW7qDgfTKaN16dqwOfTzka6ZdUZwQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAGPEXOSz0q3Jv9otJK3a5Wn7WC46kE/AlFrriKXeG1M9B82JZccou0iY+eFc/5c4j4T0lWO0/LkerD4T7nnrnIl3S89smjkvepkvmr9sZUNkoef4/9rlH2GJu9VT+k7kzbY8qKjTrIsqBXl3ZRvlPx18VRkza7Lyik744AxRlI5CH+XbEexRsgDWzuMQaISltEetXw+R7SMWKPriBJf5RYaCtWL2JGus6TswJKxJKesdYTBbwq+4hZLdq5WW2F3/MAwZGb+UhFzL/7K26csOb57yM5bvF9xJrLEObOkAAAADEqOxR/45SUw0cJ2G8hqaBKcyJKT38usi9jyDakVVo7siicD1KhFMU51CAR4tsriwPIy6Dp5eeLVrDG36nJnlP+w8iFjkJrz0cB6b95WDVDjDDDIszizCQbWxQwV0y+2b2Gbp9220KRD8/VoTSH1OX69sncpWBsbzMfaQnGQQBCAbd9uHXZaGT2cvhRs7reawctIXtX1s3kTqM9YV+/wCpDYt1Nr7FIG/NZSAJaRmaHBiO77G5QddqLJKAyyPHpWBz9dZMaFB76I7eyM43bf4c98EbUqbXp4dmJnghPiIo2AMYAAUCwFwVABwJBwoQAw4AFAAdGPjGnpHhdYfIsHgAAAAAAAAAAAAAAAAAABsjEgUNGQYDAQgPDR4FFQoIAQQLDAcOEBYaFAITEQkRFwAAHB0PxzhVJpLzJZ4DAAAAVVNE";
        
    let bytes = base64::decode(base64_string).unwrap();
    let mut send = connection.open_uni().await?;
    send.write_all(&bytes).await?;
    //send.write_all(&buffer).await?;
    //send.write_all("hello world".as_bytes()).await?;
    send.finish().await?;
    Ok(())
}
