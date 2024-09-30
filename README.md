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
  stats  Print statistics about all projects
  size   Calculate the total size of all target folders
  clean  Execute `cargo clean` on all projects
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

```
