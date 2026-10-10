#!/bin/sh
set -eu
cd "$(dirname "$0")"
if command -v cargo >/dev/null 2>&1; then
  cargo_bin=cargo
elif [ -x "$HOME/.cargo/bin/cargo" ]; then
  cargo_bin="$HOME/.cargo/bin/cargo"
else
  echo "Rust/Cargo fehlt. Installiere eine Rust-Toolchain." >&2
  exit 1
fi
"$cargo_bin" build --release --locked --offline
python3 tools/prepare-ui.py
cd qt
qmake6 kanata-studio-rust.pro
make -j2 >/dev/null
cd ..
touch target/build.stamp
