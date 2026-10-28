#!/bin/sh
set -eu
cd "$(dirname "$0")"
app_binary=qt/kanata-studio-rust
build_stamp=target/build.stamp
needs_build=false
if [ ! -x "$app_binary" ] || [ ! -f "$build_stamp" ]; then
  needs_build=true
else
  for input in Cargo.toml Cargo.lock build.sh run.sh tools/prepare-ui.py qt/*.cpp qt/*.h qt/*.pro qt/resources.qrc ui/*.qml ui/*.js; do
    if [ "$input" -nt "$build_stamp" ]; then needs_build=true; break; fi
  done
  if [ "$needs_build" = false ] && [ -n "$(find src -type f -newer "$build_stamp" -print -quit)" ]; then
    needs_build=true
  fi
fi
if [ "$needs_build" = true ]; then sh ./build.sh; fi
if [ "$#" -eq 1 ] && [ "$1" = --self-test ]; then
  exec "./$app_binary" --self-test tests/fixtures/wrapped-layout.kbd
fi
exec "./$app_binary" "$@"
