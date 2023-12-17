#!/bin/bash

# download the spec
curl -s -o flux/openapi.yaml https://raw.githubusercontent.com/RunOnFlux/fluxdocs/master/spec/openapi.yaml
# fix typos :/
sed -i 's/enviroment/environment/g' flux/openapi.yaml

# generate types
bunx @openapitools/openapi-generator-cli generate --global-property models,modelDocs=false -i flux/openapi.yaml -g rust -o src/flux_types
