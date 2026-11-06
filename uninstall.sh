#!/usr/bin/env bash
set -Eeuo pipefail

APP_NAME="rscashm"
PREFIX="${PREFIX:-$HOME/.local}"

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

run_privileged rm -f \
    "$PREFIX/bin/rscashm" \
    "$PREFIX/lib/$APP_NAME/rscashm" \
    "$PREFIX/share/applications/rscashm.desktop" \
    "$PREFIX/share/pixmaps/rscashm.png"
run_privileged rmdir "$PREFIX/lib/$APP_NAME" 2>/dev/null || true

if command -v update-desktop-database >/dev/null 2>&1; then
    run_privileged update-desktop-database "$PREFIX/share/applications" || true
fi

printf 'RScashm has been removed. User data was left untouched.\n'