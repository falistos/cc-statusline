#!/bin/sh
# cc-statusline installer
#
#   curl -fsSL https://raw.githubusercontent.com/mediavee/cc-statusline/main/install.sh | sh
#
# Environment variables:
#   INSTALL_DIR   target directory (default: $HOME/.local/bin)
#   VERSION       specific tag to install (default: latest)

set -eu

REPO="mediavee/cc-statusline"
BIN="cc-statusline"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${VERSION:-latest}"

err() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

info() { printf '%s\n' "$*"; }

uname_s=$(uname -s | tr '[:upper:]' '[:lower:]')
uname_m=$(uname -m)

case "$uname_s" in
  linux)  os_target="unknown-linux-musl" ;;
  darwin) os_target="apple-darwin" ;;
  *)      err "unsupported OS: $uname_s" ;;
esac

case "$uname_m" in
  x86_64|amd64)   arch="x86_64" ;;
  aarch64|arm64)  arch="aarch64" ;;
  *)              err "unsupported arch: $uname_m" ;;
esac

target="${arch}-${os_target}"

if [ "$VERSION" = "latest" ]; then
  tag=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
    | grep -m1 '"tag_name"' \
    | sed -E 's/.*"tag_name"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/')
  [ -n "$tag" ] || err "could not determine latest release tag"
else
  tag="$VERSION"
fi

archive="${BIN}-${tag}-${target}.tar.gz"
url="https://github.com/${REPO}/releases/download/${tag}/${archive}"

info "Downloading $url"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

if ! curl -fsSL "$url" -o "$tmp/$archive"; then
  err "failed to download $url"
fi

tar -xzf "$tmp/$archive" -C "$tmp"

mkdir -p "$INSTALL_DIR"
install -m 755 "$tmp/$BIN" "$INSTALL_DIR/$BIN"

info "Installed $BIN $tag to $INSTALL_DIR/$BIN"

case ":$PATH:" in
  *:"$INSTALL_DIR":*) ;;
  *) info "Note: $INSTALL_DIR is not in your PATH. Add it to your shell rc." ;;
esac

info "Next step: $BIN init"
