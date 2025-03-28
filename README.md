# cargo-wash

[![Crates.io](https://img.shields.io/crates/v/cargo-wash.svg)](https://crates.io/crates/cargo-wash)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/bircni/cargo-wash/blob/main/LICENSE)
[![CI](https://github.com/bircni/cargo-wash/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/bircni/cargo-wash/actions/workflows/ci.yml)

`cargo-wash` is a tool to clean up your Cargo workspaces.
It can show the size of all your target folders, and delete them if you want.

## Usage

```sh
A tool to clean up your Cargo caches

Usage: cargo-wash <COMMAND>

Commands:
  clean  Execute `cargo clean` on all projects to remove build artifacts
  stats  Print statistics about all Rust projects in the directory
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

```

## Installation

You can install `cargo-wash` using `cargo`:

```sh
cargo install cargo-wash
```

or with `cargo-binstall`:

```sh
cargo binstall cargo-wash
```
