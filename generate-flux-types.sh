#!/bin/bash

curl -s -o flux/openapi.yaml https://raw.githubusercontent.com/RunOnFlux/fluxdocs/master/spec/openapi.yaml

bunx @openapitools/openapi-generator-cli generate --global-property models,modelDocs=false -i flux/openapi.yaml -g rust -o src/flux_types
