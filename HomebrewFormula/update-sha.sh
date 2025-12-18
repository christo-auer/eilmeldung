#!/usr/bin/env bash

TAG=$(git describe --tags)
GITURL="https://github.com/christo-auer/eilmeldung/archive/refs/tags/${TAG}.tar.gz"

SHA256SUM=$(curl -L --silent "${GITURL}" | sha256sum --binary | awk '{ print $1; }')

sed -i "s/sha256 .*/sha256 \"${SHA256SUM}\"/" ./HomebrewFormula/eilmeldung.rb 
