#!/bin/bash
set -o errexit
set -o nounset
set -o pipefail

rm -f image.png download.zip
cargo run --bin populate download.zip
world=`find unzipped -name \*.mzx -o -name \*.MZX | sort -R | head -n 1`
cargo run --bin capture image.png ${world}
open image.png
