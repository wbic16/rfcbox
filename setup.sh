#!/bin/sh
# Extract the ietf.phext corpus published in this repo (data/) and verify it.
# The phext ships as an xz stream split into parts under GitHub's 100 MB blob
# limit (phbot storage precedent); this script reassembles, extracts, and
# SHA-256-verifies every artifact. Run from anywhere: ./setup.sh
set -eu
cd "$(dirname "$0")"

cat data/ietf.phext.xz.part-* | xz -d > ietf.phext
xz -dc data/ietf-source.json.xz > ietf-source.json
cp data/ietf.index.json ietf.index.json

sha256sum -c data/SHA256SUMS
echo "ietf.phext corpus extracted and verified in $(pwd)"
