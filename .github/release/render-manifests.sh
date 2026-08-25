#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?version is required}"
TAG="${2:?tag is required}"
REPOSITORY="${3:?repository is required}"
DIST="${4:?dist directory is required}"
OUTPUT="${GITHUB_WORKSPACE:-$(pwd)}/generated"
BASE_URL="https://github.com/${REPOSITORY}/releases/download/${TAG}"

sha() { sha256sum "$DIST/$1" | awk '{print $1}'; }
render() {
  sed \
    -e "s|@VERSION@|${VERSION}|g" \
    -e "s|@REPOSITORY@|${REPOSITORY}|g" \
    -e "s|@MACOS_ARM64_URL@|${BASE_URL}/ptd-cli-${TAG}-aarch64-apple-darwin.tar.gz|g" \
    -e "s|@MACOS_ARM64_SHA@|$(sha "ptd-cli-${TAG}-aarch64-apple-darwin.tar.gz")|g" \
    -e "s|@MACOS_X86_64_URL@|${BASE_URL}/ptd-cli-${TAG}-x86_64-apple-darwin.tar.gz|g" \
    -e "s|@MACOS_X86_64_SHA@|$(sha "ptd-cli-${TAG}-x86_64-apple-darwin.tar.gz")|g" \
    -e "s|@LINUX_ARM64_URL@|${BASE_URL}/ptd-cli-${TAG}-aarch64-unknown-linux-gnu.tar.gz|g" \
    -e "s|@LINUX_ARM64_SHA@|$(sha "ptd-cli-${TAG}-aarch64-unknown-linux-gnu.tar.gz")|g" \
    -e "s|@LINUX_X86_64_URL@|${BASE_URL}/ptd-cli-${TAG}-x86_64-unknown-linux-gnu.tar.gz|g" \
    -e "s|@LINUX_X86_64_SHA@|$(sha "ptd-cli-${TAG}-x86_64-unknown-linux-gnu.tar.gz")|g" \
    -e "s|@WINDOWS_X86_64_URL@|${BASE_URL}/ptd-cli-${TAG}-x86_64-pc-windows-msvc.zip|g" \
    -e "s|@WINDOWS_X86_64_SHA@|$(sha "ptd-cli-${TAG}-x86_64-pc-windows-msvc.zip")|g" \
    "$1" > "$2"
}

mkdir -p "$OUTPUT"
render .github/release/ptd-cli.rb.in "$OUTPUT/ptd-cli.rb"
render .github/release/ptd-cli.json.in "$OUTPUT/ptd-cli.json"
jq empty "$OUTPUT/ptd-cli.json"
