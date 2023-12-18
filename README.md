# tui-term
[![Crates](https://img.shields.io/crates/v/tui-term?style=flat-square)](https://crates.io/crates/tui-term)
[![Documentation](https://img.shields.io/badge/tui_term-documentation-fc0060?style=flat-square)](https://docs.rs/tui-term)
[![Matrix Chat Room](https://img.shields.io/badge/chat-on%20matrix-1d7e64?logo=matrix&style=flat-square)](https://matrix.to/#/#tui-term-main:matrix.org)

A pseudoterminal widget for the  [ratatui](https://github.com/tui-rs-revival/ratatui) crate.

![Demo of tui-term](https://vhs.charm.sh/vhs-4zK1zlTOSueAmlOkZlssBr.gif)

## Status

> [!NOTE]
> This project is currently in active development and should be considered a work in progress.
> The goal of tui-term is to provide a robust and well-tested pseudoterminal widget for users of the `ratatui` crate.

## Installation

To use `tui-term`, simply add it as a dependency in your Cargo.toml file:

```
[dependencies]
tui-term = "0.1.6"
```
or use `cargo add`:
```
cargo add tui-term
```

## Examples

Check out the examples directory, for more information, or run an example:
```
cargo run --example simple_ls_rw
```

## Chat Room
Join our matrix chat room, for possibly synchronous communication.

## Architecture

For a top-level understanding of the architecture of `tui-term` and the design choices made, please refer to the [Architecture](docs/ARCHITECTURE.md) document.

## Contributing
We welcome contributions from the community! If you're interested in contributing to tui-term, please refer to the contribution guidelines for instructions on how to get started.

[How to contribute.](./docs/CONTRIBUTING.md)

## Changes
[Changelog](./CHANGELOG.md)

## License
MIT
