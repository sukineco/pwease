#!/bin/sh

[ -z "$MODE" ] && MODE="release"
[ -z "$TARGET" ] && TARGET="x86_64-unknown-linux-musl"
[ -z "$PREFIX" ] && PREFIX="/usr/local"

run_root() {
  if [ "$(whoami)" = "root" ]; then
    "$@"
  elif command -v sudo >/dev/null 2>&1; then
    sudo "$@"
  elif command -v doas >/dev/null 2>&1; then
    doas "$@"
  elif command -v pwease >/dev/null 2>&1; then
    pwease "$@"
  elif command -v su >/dev/null 2>&1; then
    su -c "$*"
  else
    echo "No user substitution program found and script is not ran by root."
  fi
}

BUILT_BIN="target/${TARGET}/${MODE}/pwease"

echo "\$ cargo build --${MODE} --target ${TARGET}"
cargo build --${MODE} --target ${TARGET}

echo "# install --group=0 --owner=0 --mode=4755 -t ${PREFIX}/bin ${BUILT_BIN}"
run_root install --group=0 --owner=0 --mode=4755 -t ${PREFIX}/bin ${BUILT_BIN}
