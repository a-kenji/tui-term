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

To use `tui-term`, simply add it as a dependency in your `Cargo.toml` file:

```sh
[dependencies]
tui-term = "0.2.0"
```
or use `cargo add`:
```sh
cargo add tui-term
```

## Examples

Check out the examples directory, for more information, or run an example:
```sh
cargo run --example simple_ls_rw
```


## Controller

The controller is an `experimental` feature helping with managing the lifecycle of commands that are spawned inside a pseudoterminal.
Currently the support is limited to oneshot commands.

To activate the feature:
```sh
cargo add tui-term -F unstable
```

## Chat Room
Join our [matrix chat room](https://matrix.to/#/#tui-term-main:matrix.org), for possibly synchronous communication.

## Architecture

For an overview of `tui-term`'s architecture and design principles, please refer to the [Architecture](docs/ARCHITECTURE.md) documentation.

## Contributing
We welcome contributions from the community!
Check out the [Contributing Guidelines](./docs/CONTRIBUTING.md) on how to get started.

## Release Notes
Stay updated with the latest changes by viewing the [Changelog](./CHANGELOG.md).

## License
`tui-term` is available under the MIT license. See [LICENCE](LICENSE) for more information.
