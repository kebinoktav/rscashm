#!/usr/bin/env bash
set -Eeuo pipefail

APP_NAME="rscashm"
PREFIX="${PREFIX:-$HOME/.local}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

usage() {
    printf 'Usage: %s [--icon /path/to/icon.png]\n' "${0##*/}"
    printf 'Default install prefix: %s (override with PREFIX=/path)\n' "$PREFIX"
}

if [[ $# -eq 0 ]]; then
    ICON_PATH="$SCRIPT_DIR/rscashm.png"
elif [[ $# -eq 1 && $1 == "--help" ]]; then
    usage
    exit 0
elif [[ $# -eq 2 && $1 == "--icon" ]]; then
    ICON_PATH=$2
elif [[ $# -eq 1 ]]; then
    ICON_PATH=$1
else
    usage
    exit 2
fi

if [[ ! -f $ICON_PATH || ! -r $ICON_PATH ]]; then
    printf 'Error: icon file is missing or unreadable: %s\n' "$ICON_PATH" >&2
    exit 1
fi

ICON_PATH="$(cd -- "$(dirname -- "$ICON_PATH")" && pwd -P)/$(basename -- "$ICON_PATH")"
PNG_SIGNATURE="$(od -An -tx1 -N8 -- "$ICON_PATH" | tr -d '[:space:]')"
if [[ $PNG_SIGNATURE != "89504e470d0a1a0a" ]]; then
    printf 'Error: the selected icon is not a valid PNG file.\n' >&2
    exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
    printf 'Error: Cargo is required. Install Rust first: https://rustup.rs\n' >&2
    exit 1
fi

run_privileged() {
    if [[ $EUID -eq 0 ]]; then
        "$@"
    elif [[ -w $PREFIX || ( ! -e $PREFIX && -w "$(dirname -- "$PREFIX")" ) ]]; then
        "$@"
    elif command -v sudo >/dev/null 2>&1; then
        sudo "$@"
    else
        printf 'Error: cannot write under %s; choose a writable PREFIX or install sudo.\n' "$PREFIX" >&2
        return 1
    fi
}

desktop_quote() {
    local value=$1
    value=${value//\\/\\\\}
    value=${value//\"/\\\"}
    value=${value//\`/\\\`}
    value=${value//\$/\\\$}
    printf '"%s"' "$value"
}

printf 'Building %s in release mode...\n' "$APP_NAME"
cargo build --release --locked --manifest-path "$SCRIPT_DIR/Cargo.toml"

BINARY_PATH="$SCRIPT_DIR/target/release/rscashm"
if [[ ! -x $BINARY_PATH ]]; then
    printf 'Error: release binary was not created: %s\n' "$BINARY_PATH" >&2
    exit 1
fi

DESKTOP_TMP="$(mktemp)"
trap 'rm -f -- "$DESKTOP_TMP"' EXIT
DESKTOP_EXEC="$(desktop_quote "$PREFIX/bin/rscashm")"
DESKTOP_ICON="$PREFIX/share/pixmaps/rscashm.png"
while IFS= read -r line || [[ -n $line ]]; do
    case $line in
        Exec=*) printf 'Exec=%s\n' "$DESKTOP_EXEC" ;;
        TryExec=*) printf 'TryExec=%s\n' "$PREFIX/bin/rscashm" ;;
        Icon=*) printf 'Icon=%s\n' "$DESKTOP_ICON" ;;
        *) printf '%s\n' "$line" ;;
    esac
done < "$SCRIPT_DIR/rscashm.desktop" > "$DESKTOP_TMP"

printf 'Installing %s under %s...\n' "$APP_NAME" "$PREFIX"
run_privileged install -d -m 755 \
    "$PREFIX/bin" \
    "$PREFIX/lib/$APP_NAME" \
    "$PREFIX/share/applications" \
    "$PREFIX/share/pixmaps"
run_privileged install -m 755 "$BINARY_PATH" "$PREFIX/lib/$APP_NAME/rscashm"
run_privileged install -m 755 "$SCRIPT_DIR/rscashm-launcher.sh" "$PREFIX/bin/rscashm"
run_privileged install -m 644 "$DESKTOP_TMP" "$PREFIX/share/applications/rscashm.desktop"
run_privileged install -m 644 "$ICON_PATH" "$PREFIX/share/pixmaps/rscashm.png"

if command -v update-desktop-database >/dev/null 2>&1; then
    run_privileged update-desktop-database "$PREFIX/share/applications" || true
fi

printf '\n%s is installed. Find it in your app menu or run: %s/bin/rscashm\n' "$APP_NAME" "$PREFIX"
printf 'App data is stored separately for each user under ${XDG_DATA_HOME:-$HOME/.local/share}/%s\n' "$APP_NAME"