#!/bin/bash

set -e

dfx deploy ic_side_services_backend --ic --argument '(variant { mainnet })'
