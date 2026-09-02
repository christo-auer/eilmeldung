#!/usr/bin/env zsh

CLONE_DIR="/tmp/base16-schemes"

git clone https://github.com/tinted-theming/schemes "${CLONE_DIR}"

rsync -av --delete "${CLONE_DIR}/base16/" assets/base16-schemes/
cp "${CLONE_DIR}/LICENSE" assets/base16-schemes

rm -rf "${CLONE_DIR}"
