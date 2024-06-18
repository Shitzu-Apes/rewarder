#!/bin/bash
set -e
cd "`dirname $0`"

cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/*.wasm ./res/

wasm-opt -O4 res/rewarder.wasm -o res/rewarder.wasm --strip-debug --vacuum
wasm-opt -O4 res/memeseason.wasm -o res/memeseason.wasm --strip-debug --vacuum
