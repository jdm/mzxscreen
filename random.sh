#!/bin/bash
set -o errexit
set -o nounset
set -o pipefail


if [ -z "${1+x}" ]
  then
    rm -f image.png download.zip
    cargo run --bin populate download.zip output.json
fi
find unzipped \( -name \*.zip -o -name \*.ZIP \) -exec unzip -d unzipped {} \;
world=`find unzipped -name \*.mzx -o -name \*.MZX | sort -R | head -n 1`
cargo run --bin capture image.png output.json "${world}"
open image.png
