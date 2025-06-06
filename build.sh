#!/bin/bash
set -e
cd "`dirname $0`"

cd crates/memeseason
cargo near build reproducible-wasm
cd ../..
cp target/near/memeseason/memeseason.wasm ./res/

cd crates/memeseason
cargo near build non-reproducible-wasm --features=integration-test
cd ../..
cp target/near/memeseason/memeseason.wasm ./res/memeseason_integration_test.wasm

cd crates/rewarder
cargo near build reproducible-wasm
cd ../..
cp target/near/rewarder/rewarder.wasm ./res/
