#!/bin/bash

# download the spec
curl -s -o flux/openapi.yaml https://raw.githubusercontent.com/RunOnFlux/fluxdocs/master/spec/openapi.yaml
# fix typos :/
sed -i 's/enviroment/environment/g' flux/openapi.yaml
sed -i '18102s/string/number/g' flux/openapi.yaml
sed -i '19806a\                  format: int64' flux/openapi.yaml
sed -i '19899a\                  format: int64' flux/openapi.yaml
sed -i 's/appSpecifications/appSpecification/g' flux/openapi.yaml

# generate types
bunx @openapitools/openapi-generator-cli generate --global-property models,modelDocs=false -i flux/openapi.yaml -g rust -o src/flux_types
