#!/bin/bash
set -o errexit
set -o nounset
set -o pipefail

if [ -z "$1" ]
  then
    rm -f image.png download.zip
    cargo run --bin populate download.zip output.json
fi
IFS=$'\n'
for archive in "`find unzipped -name \*.zip -o -name \*.ZIP`"; do
    if [ -n "${archive}" ]
      then
        echo ${archive}
        unzip -d unzipped "${archive}"
    fi
done
world=`find unzipped -name \*.mzx -o -name \*.MZX | sort -R | head -n 1`
cargo run --bin capture image.png "${world}"
open image.png
