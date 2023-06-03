[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE-APACHE)

```sh
# -extensions v3_req generates with basicConstraint: CA:false (see /etc/pki/tls/openssl.cnf)
openssl req -x509 -newkey ed25519 -days 365 -nodes -keyout key.pem -out cert.pem -subj "/CN=localhost" -addext "subjectAltName=DNS:localhost,IP:127.0.0.1" -extensions v3_req

cp cert.pem key.pem $FIREDANCER_DIR

# start fd_quic inside firedancer repo
./build/linux/gcc/x86_64/unit-test/test_quic_server --src-mac 00:00:00:00:00:00 --ssl-cert cert.pem --ssl-key key.pem --listen-port 9000

# start the client
RUST_LOG=trace cargo run --example minimal_client
```
