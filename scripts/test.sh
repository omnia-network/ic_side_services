#!/bin/bash

set -e

# integration tests
./scripts/download-pocket-ic.sh

./scripts/build-test-canister.sh

POCKET_IC_MUTE_SERVER=1 POCKET_IC_BIN="$(pwd)/bin/pocket-ic" cargo t
