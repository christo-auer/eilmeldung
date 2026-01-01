#!/usr/bin/env bash


if [ "$1" = "somearg" ]; then
  echo secret
  exit 0
fi

if [ "$1" = "somearg" -a "$2" = "someotherarg" ]; then
  echo
  echo secret
  echo
  echo
  exit 1
fi

if [ "$1" = "somerror" ]; then
  echo "this is expected" 1>&2
  exit 1
fi
