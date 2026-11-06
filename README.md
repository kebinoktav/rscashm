
# RSCashM (Rust Simple Cash Manager)

> **A simple and direct cash manager for tracking income and expenses.**

**RSCashM is currently in beta.**

RSCashM is a simple cash management application written entirely in Rust. It is designed for people who want to track their personal income and expenses without dealing with the complexity of full accounting software.

The goal is simple: **record your money, see your transactions, and keep track of your cash.**

## Features

* Simple graphical interface using `iced`
* Track income and expenses
* Ledger data stored in binary files using `bincode`
* Theme, selected ledger, custom colors, and wallpaper stored together in `config.json`
* File selection using `rfd`
* Data serialization using `serde`
* Written entirely in Rust

## Why RSCashM?

RSCashM focuses on **simple and direct cash management** rather than full accounting.

If you only want to answer questions like:

* How much money do I have?
* Where did my money go?
* How much did I receive?
* What did I spend?

you shouldn't need to learn a complete accounting system just to record a few transactions.

RSCashM aims to keep that process simple.

## Dependencies

| Purpose       | Crate            |
| ------------- | ---------------- |
| GUI           | `iced`           |
| Serialization | `serde`          |
| Ledger data   | `bincode`        |
| Settings      | `serde_json`     |
| File Dialog   | `rfd`            |

## Requirements

* Rust 1.92 or newer
* Cargo

Install Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Or visit:

https://rustup.rs/

Check your installation:

```bash
rustc --version
cargo --version
```

## Build

Clone the repository:

```bash
git clone https://github.com/kebinoktav/rscashm.git
cd rscashm
```

Build and run:

```bash
cargo run --release
```

Build only:

```bash
cargo build --release
```

The binary will be generated at:

```text
target/release/rscashm
```

## Installation

Run directly:

```bash
./target/release/rscashm-iced
```

Install the application and its desktop entry with the bundled icon:

```bash
./install.sh
```

The installer builds the release binary and installs it under `~/.local` by default, so a normal user installation does not need `sudo`. It uses the repository's `rscashm.png` icon automatically. Set `PREFIX=/usr/local` to install system-wide (the script uses `sudo` if needed), or pass `--icon /path/to/icon.png` to choose another PNG. Remove the installed files with `./uninstall.sh`; use the same `PREFIX` value for a system-wide installation.

The installer also creates a desktop entry whose launcher and icon paths match the selected `PREFIX`, so it works even when the desktop session does not include `~/.local/bin` in `PATH`. No username is hardcoded: each user launches the same installed program with their own `$HOME` and `${XDG_DATA_HOME:-~/.local/share}` data directory. It requires Bash, Cargo, and standard Linux `install` utilities; `update-desktop-database` is optional, so the script works on Arch Linux and other common Linux distributions.

When launched from the desktop entry, `config.json` is stored in `${XDG_DATA_HOME:-~/.local/share}/rscashm/`. It contains the selected ledger path, theme, custom colors, and wallpaper path. Ledger transaction files remain separate `.bin` files. Existing `cfg.txt`, `theme.txt`, and `appearance.json` settings are read as a fallback and then saved to `config.json`.

## Current Status

RSCashM is currently in **beta**.

The project is still under development, and features, behavior, and the user interface may change.

Bug reports, suggestions, and feedback are welcome.

## License

Licensed under the Mozilla Public License 2.0.
