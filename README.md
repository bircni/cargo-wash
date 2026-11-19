# cargo-wash

[![Crates.io](https://img.shields.io/crates/v/cargo-wash.svg)](https://crates.io/crates/cargo-wash)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/bircni/cargo-wash/blob/main/LICENSE)
[![CI](https://github.com/bircni/cargo-wash/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/bircni/cargo-wash/actions/workflows/ci.yml)

`cargo-wash` is a tool to clean up your Cargo workspaces.
It finds all Rust projects below a directory, shows the size of their target folders,
deletes them if you want, and can run cargo commands in every project.

## Usage

```sh
A tool to clean up your Cargo caches

Usage: cargo wash [OPTIONS] <COMMAND>

Commands:
  stats   Print statistics about all Rust projects in the directory
  clean   Clean the `target` folders of all Rust projects in the directory
  build   Execute `cargo build` on all Rust projects in the directory
  check   Execute `cargo check` on all Rust projects in the directory
  doc     Execute `cargo doc` on all Rust projects in the directory
  run     Execute `cargo run` on all Rust projects in the directory
  test    Execute `cargo test` on all Rust projects in the directory
  bench   Execute `cargo bench` on all Rust projects in the directory
  update  Execute `cargo update` on all Rust projects in the directory
  help    Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  Show more output, use twice for even more
  -q, --quiet       Only show warnings and errors
  -h, --help        Print help
  -V, --version     Print version
```

### Examples

```sh
# Show the biggest target folders below ~/code, searching three levels deep
cargo wash stats --path ~/code --depth 3 --sort size

# Preview what cleaning every target folder of at least 1 GB would free
cargo wash clean --path ~/code --depth 3 --min-size 1GB --dry-run

# Clean without being asked, skipping some projects
cargo wash clean --yes --exclude my-project,'vendor/*'

# Run `cargo update` in four projects at a time and keep a log per project
cargo wash update --jobs 4 --log-dir logs

# Pass arguments on to cargo
cargo wash build -- --release

# Use the sizes in a script
cargo wash stats --json | jq '.total_size'
```

### Finding projects

All commands share the options that select the projects:

| Option | Description |
| --- | --- |
| `-p, --path <PATH>` | Directory to search, can be given multiple times. Defaults to the current directory. |
| `-d, --depth <DEPTH>` | How many directory levels below each path are searched. Defaults to `1`, `0` only checks the paths themselves. |
| `-e, --exclude <PATTERN>` | Skip folders whose name or path relative to the search path matches the glob pattern. Can be given multiple times or comma-separated. |

A folder containing a `Cargo.toml` is a project, and the search does not continue inside of it:
the members of a workspace share the target folder of the workspace root.
Hidden folders, `target`, `node_modules` and symlinks are never searched.

The target folder is `target` inside the project, or the folder set in `CARGO_TARGET_DIR`.

### Cleaning

`clean` runs `cargo clean` in every project. It lists the projects and asks for confirmation first,
pass `--yes` to skip the question (required in scripts) or `--dry-run` to only see the list.

### Running cargo commands

Projects are processed one after another, `--jobs <N>` runs the command in `N` projects at the same time.
The output of a failing command is shown, use `-v` to also see the output of successful ones
or `--log-dir <DIR>` to write it to one file per project.
`cargo wash` exits with a non-zero status if the command failed in any project.

## Installation

You can install `cargo-wash` using `cargo`:

```sh
cargo install cargo-wash
```

or with `cargo-binstall`:

```sh
cargo binstall cargo-wash
```
