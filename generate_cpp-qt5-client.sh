#!/usr/bin/env bash
set -euo pipefail

# Change into directory where this shell script is located
SCRIPT_ROOT=$(cd -P -- "$(dirname -- "$0")" && pwd -P)
cd "${SCRIPT_ROOT}"

OPENAPI_GENERATOR_TARGET=cpp-qt5-client
OPENAPI_GENERATOR_CLI_TAG=cli-latest
#OPENAPI_GENERATOR_CLI_TAG=cli-5.0.x
#OPENAPI_GENERATOR_CLI_TAG=cli-4.2.x

#OPENAPI_GENERATOR_VERSION=4.1.1

#if [ ! -f openapi-generator-cli.jar ];
#then
#    wget http://central.maven.org/maven2/org/openapitools/openapi-generator-cli/${OPENAPI_GENERATOR_VERSION}/openapi-generator-cli-${OPENAPI_GENERATOR_VERSION}.jar -O openapi-generator-cli.jar
#fi

#java -jar openapi-generator-cli.jar generate -g cpp-qt5-client -i resources/openapi.yaml --generate-alias-as-model

podman --root /tmp run --rm \
-v ${PWD}:/local:Z \
openapitools/openapi-generator:${OPENAPI_GENERATOR_CLI_TAG} \
generate \
-i /local/resources/openapi.yaml \
-g ${OPENAPI_GENERATOR_TARGET} \
-o /local/resources/${OPENAPI_GENERATOR_TARGET} \
--generate-alias-as-model \
