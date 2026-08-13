# RSCashM (Rust Simple Cash Manager)

RSCashM is a simple cash management application written in Rust.

It uses `eframe` and `egui` for the graphical interface, with a focus on simplicity and performance.

## Features

- GUI application using `eframe` (`egui`)
- Binary data storage using `bincode`
- File selection support using `rfd`
- Data serialization using `serde`
- Written entirely in Rust

## Dependencies

| Purpose | Crate |
|---------|-------|
| GUI | `eframe`, `egui` |
| Serialization | `serde` |
| Data Storage | `bincode` |
| File Dialog | `rfd` |

## Requirements

- Rust 1.92 or newer
- Cargo

Install Rust:

https://rustup.rs/

Check installation:

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

## License

Licensed under the Mozilla Public License 2.0.

See [LICENSE](LICENSE) for details.

## Repository

https://github.com/kebinoktav/rscashm
