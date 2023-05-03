#!/bin/sh

echo 'Start building images...'

set -eux;

docker build --build-arg GIT_REVISION=$(git rev-parse HEAD) -t nintendo-shop-backend/all-in-one -f ./deploy/all-in-one/Dockerfile .