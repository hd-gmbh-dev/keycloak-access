#!/bin/bash

cargo set-version --workspace $1

git add .
git commit -m "build: prepare release v$1"
git push
# git tag v$1
# git push -u origin v$1

pnpm publish --recursive --access public --no-git-checks
cargo publish -p wfrs-model
cargo publish -p wfrs-validator
cargo publish -p wfrs-engine
