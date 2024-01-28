#!/bin/bash

# build the rust to wasm
cargo build --target wasm32-unknown-unknown --release

# copy the wasm file to the web directory
cp ./target/wasm32-unknown-unknown/release/rolly-polly.wasm ./web/rolly-polly.wasm