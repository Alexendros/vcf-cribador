#!/bin/sh
# scripts/version.sh — lee y muestra la versión desde Cargo.toml

set -eu

VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo "$VERSION"
