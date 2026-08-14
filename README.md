
# RSCashM (Rust Simple Cash Manager)

> **A simple and direct cash manager for tracking income and expenses.**

**RSCashM is currently in beta.**

RSCashM is a simple cash management application written entirely in Rust. It is designed for people who want to track their personal income and expenses without dealing with the complexity of full accounting software.

The goal is simple: **record your money, see your transactions, and keep track of your cash.**

## Features

* Simple graphical interface using `eframe` and `egui`
* Track income and expenses
* Local data storage
* Binary data storage using `bincode`
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
| GUI           | `eframe`, `egui` |
| Serialization | `serde`          |
| Data Storage  | `bincode`        |
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
./target/release/rscashm
```

Or install manually:

```bash
sudo install target/release/rscashm /usr/bin/rscashm
```

## Current Status

RSCashM is currently in **beta**.

The project is still under development, and features, behavior, and the user interface may change.

Bug reports, suggestions, and feedback are welcome.

## License

Licensed under the Mozilla Public License 2.0.
