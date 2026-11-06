#!/usr/bin/env bash
set -Eeuo pipefail

APP_NAME="rscashm"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
PREFIX="$(cd -- "$SCRIPT_DIR/.." && pwd -P)"
APP_DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/$APP_NAME"
APP_BINARY="$PREFIX/lib/$APP_NAME/rscashm"

if [[ ! -x $APP_BINARY ]]; then
    printf 'RScashm is not installed. Run install.sh from the project directory.\n' >&2
    exit 1
fi

mkdir -p -- "$APP_DATA_DIR"
cd -- "$APP_DATA_DIR"
exec "$APP_BINARY" "$@"