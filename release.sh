#!/bin/bash

cargo set-version --workspace $1
cargo build
git add .
git commit -m "build: prepare release v$1"
git push
# git tag v$1
# git push -u origin v$1

cargo publish -p keycloak-access
