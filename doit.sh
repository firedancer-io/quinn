#!/bin/bash

cargo build --example client_minimal
RUST_LOG=trace sudo -E nsenter --net=/var/run/netns/veth_test_xdp_1 ./target/debug/examples/client_minimal
