#!/bin/bash
set -o errexit
set -o nounset
set -o pipefail
export RUST_BACKTRACE=1

if [ -z "${1+x}" ]
  then
    rm -f image.png download.zip
    cargo run --bin populate download.zip output.json
else
  shift
fi
find unzipped \( -name \*.zip -o -name \*.ZIP \) -exec unzip -d unzipped {} \;
world=`find unzipped -name \*.mzx -o -name \*.MZX | sort -R | head -n 1`
echo cargo run --bin capture image.png output.json "${world}" "$@"
cargo run --bin capture image.png output.json "${world}" "$@"
open image.png
