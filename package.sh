#!/bin/sh
set -eu
cd "$(dirname "$0")"
sh ./build.sh
exec python3 tools/package-linux.py "$@"
