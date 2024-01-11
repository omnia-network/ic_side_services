#!/bin/bash

set -e

./scripts/download-pocket-ic.sh

./scripts/build-test-canister.sh

POCKET_IC_MUTE_SERVER=1 \
  POCKET_IC_BIN="$(pwd)/bin/pocket-ic" \
  TEST_CANISTER_WASM_PATH="$(pwd)/bin/test_canister.wasm" \
  cargo test --test integration_test
