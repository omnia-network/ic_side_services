#!/bin/bash

set -e

dfx deploy ic_side_services_backend --ic --argument '(variant { mainnet })'

# make the canister fetch the ECDSA public key and store in its state
echo -e "\nSetting up ECDSA public key in the canister..."
dfx canister call --ic ic_side_services_backend set_canister_public_key

# log addresses to see if everything went well
echo -e "\nZelId and ZelCash addresses on canister:"
dfx canister call --ic ic_side_services_backend get_addresses --query

# just log the status (controllers, balance, etc.)
echo -e "\nFetching canister status..."
dfx canister status ic_side_services_backend --ic

echo -e "\nDone!"
