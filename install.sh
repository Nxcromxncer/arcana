#!/usr/bin/env sh
# Build Arcana in release mode and install the `arcana` command.
# Usage: ./install.sh            (installs to ~/.local/bin)
#        PREFIX=/usr/local ./install.sh   (installs to /usr/local/bin)
set -eu
command -v cargo >/dev/null 2>&1 || { echo "error: Rust is not installed (get it at https://rustup.rs)" >&2; exit 1; }
cargo build --release
dest="${PREFIX:-$HOME/.local}/bin"
mkdir -p "$dest"
install -m 755 target/release/arcana "$dest/arcana"
echo "Installed: $dest/arcana"
case ":$PATH:" in *":$dest:"*) ;; *) echo "note: add $dest to your PATH to run 'arcana' from anywhere" ;; esac
echo "Try it:  arcana run examples/hello.arc"
