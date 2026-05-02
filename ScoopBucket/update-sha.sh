#!/usr/bin/env bash

if [ -z "$1" ]; then
  echo "missing argument: tag expected"
  exit 1
fi

TAG="$1"
GITURL="https://github.com/christo-auer/eilmeldung/releases/download/${TAG}/eilmeldung-x86_64-pc-windows-msvc-${TAG}.tar.gz"

SHA256SUM=$(curl -L --silent "${GITURL}" | sha256sum --binary | awk '{ print $1; }')

sed -i "s/\"hash\": .*/\"hash\": \"${SHA256SUM}\",/" ./ScoopBucket/eilmeldung.json
