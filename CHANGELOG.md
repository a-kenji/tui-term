# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0] - 2024-10-26

### Testing

- Add tests to crate so that they can be run on build systems

### Miscellaneous Tasks

- *(deps)* Bump bytes from 1.7.2 to 1.8.0
- *(deps)* Bump once_cell from 1.20.1 to 1.20.2
- *(deps)* Bump cachix/install-nix-action from V27 to 28
- *(deps)* Bump insta from 1.39.0 to 1.40.0
- *(deps)* Bump ratatui from 0.28.0 to 0.28.1
- *(deps)* Bump ratatui from 0.28.1 to 0.29.0

## [0.1.13] - 2024-08-07

### Bug Fixes

- *(uncategorized)* Changelog entry

### Refactor

- *(uncategorized)* Remove deprecated `buf.get_mut()` function
- *(uncategorized)* Prefer `area()` to `size()`

### Miscellaneous Tasks

- *(deps)* Bump bytes from 1.6.0 to 1.7.1
- *(deps)* Bump tokio from 1.38.0 to 1.39.2
- *(deps)* Bump DeterminateSystems/update-flake-lock from 22 to 23
- *(uncategorized)* Update release version
- *(uncategorized)* Update ratatui and crossterm
- *(uncategorized)* Update rust-toolchain -> `1.80.0`
- *(uncategorized)* Format `.deny.toml`
- *(uncategorized)* Move treefmt.toml until upstream fix is in
- *(uncategorized)* Fix typo in a comment

### Continuous Integration

- *(uncategorized)* Remove now obsolete audit check action
- *(uncategorized)* Update darwin runners

### Flake

- *(uncategorized)* Remove now obsolete rust-overlay.flake-utils follows

### Flake.lock

- *(uncategorized)* Update
- *(uncategorized)* Update

### Tools

- *(uncategorized)* Update cargo deny configuration to v2

## [0.1.13] - 2024-08-07

Compatible with `ratatui`: `v0.28.0`.

### Refactor

- Remove deprecated `buf.get_mut()` function
- Prefer `area()` to `size()`

### Miscellaneous Tasks

- *(uncategorized)* Update ratatui and crossterm to latest released versions
- *(uncategorized)* Move treefmt.toml until upstream fix is in

### Continuous Integration

- *(uncategorized)* Remove now obsolete audit check action
- *(uncategorized)* Update darwin runners

## [0.1.12] - 2024-06-25

Compatible with `ratatui`: `v0.27.0`.

### Bug Fixes

- *(example/smux)* Don't crash on some control codes

### Miscellaneous Tasks

- *(deps)* Bump ratatui from 0.26.3 to 0.27.0
- *(deps)* Bump tokio from 1.37.0 to 1.38.0

## [0.1.11] - 2024-05-04

Compatible with `ratatui`: `v0.26.2`.

While not a breaking change, consumers that implement the `Screen`
trait may want to look into how the underlying implementation changed in #182.

### Bug Fixes

- Render cells correctly that have no content but styles

### Continuous Integration

- *(fix)* Correct the buildbot base branch

### Examples

- Show cursor on focused pane in the `smux` example
- `smux` handle control characters

## [0.1.10] - 2024-04-16

Compatible with `ratatui`: `v0.26.2`.

MSRV is now `rustc --version`: `1.74.0`.

### Miscellaneous Tasks

- *(deps)* Bump ratatui from 0.26.1 to 0.26.2
-  Bump MSRV `1.70.0` -> `1.74.0` [**breaking**]

### Bench

-  Add divan back

## [0.1.9] - 2024-03-28

Compatible with `ratatui`: `v0.26.1`.

This release increases `tui-term`'s flexibility.

The two notable changes are breaking changes:
- Looser coupling with `vt100` https://github.com/a-kenji/tui-term/pull/152
It is now possible to implement the `Screen` trait for alternative backend parser implementations.
    -> PseudoTerminal now contains a generic.
    -> Automatic dereferencing no longer works for the constructor e.g. `PseudoTerminal::new(&parser.screen())` will fail.
- `ratatui`'s default features are not activated anymore, allowing for easier composition 
with backends other than `crossterm`

### Features

-  Add `vt100` as optional, but enabled by default

### Bug Fixes

-  Specify required feature for controller example

### Documentation

- Add doc comments to `Screen` and `Cell` trait
- *(readme)* Improve the readme

### Testing

- Use fixed crane `nextest` command

### Miscellaneous Tasks

- Move `vt100` specific details to own module
- Move `vt100` to `ratatui` cell conversion to helper
- *(toolchain)* Bump default development toolchain

### Flake.lock

- Update

## [0.1.8] - 2024-02-14

Compatible with `ratatui`: `v0.26.1`.

- *(deps)* Bump ratatui from 0.26.0 to 0.26.1
- *(inputs)* switch to nixpkgs

## [0.1.7] - 2024-02-02

Compatible with `ratatui`: `v0.26.0`.

### Features

- Add unstable controller interface, for `oneshot` commands. 
  Please see the examples for more details.
- Set visibility of the cursor

### Miscellaneous Tasks

- *(deps)* Bump cachix/install-nix-action from 24 to 25
- *(deps)* Bump cachix/cachix-action from 13 to 14
- *(deps)* Bump tokio from 1.35.0 to 1.35.1

## [0.1.6] - 2023-12-18

Compatible with `ratatui`: `v0.25.0`.

### Miscellaneous Tasks

- *(deps)* Bump ratatui from `0.24.0` to `0.25.0` [**breaking**]
- *(doc)* Fix note style
- *(doc)* Fix typo

### Bench

- Init divan

### Examples

-  Improve cross platform support

## [0.1.5] - 2023-10-25

Compatible with `ratatui`: `v0.24.0`.

MSRV is now `rustc --version`: `1.70.0`.

### Miscellaneous Tasks

- *(deps)* Bump ratatui from 0.23.0 to 0.24.0 [**breaking**]
- *(fmt)* init taplo

### Continuous Integration

- *(uncategorized)* Init merge queue

## [0.1.4] - 2023-08-28

Compatible with `ratatui`: `v0.23.0`.

### Miscellaneous Tasks

- *(deps)* Update ratatui `v0.22.0` -> `v0.23.0` [**breaking**]
- *(docs)* Document clippy version
- *(chore)* Use nightly formatter

## [0.1.3] - 2023-08-25

### Bug Fixes

- *(uncategorized)* Boundary condition check for the cursor

### Miscellaneous Tasks

- *(ci)* Adjust automatic updates
- *(deps)* Bump tokio from 1.29.1 to 1.32.0
- *(deps)* Bump crossterm from 0.26.1 to 0.27.0
- *(deps)* Bump tokio from 1.28.2 to 1.29.1
- *(uncategorized)* Document the release process
- *(uncategorized)* Bump ratatui `0.21.0` -> `0.22.0`
- *(uncategorized)* Rename PseudoTerm to PseudoTerminal [**breaking**]

### Continuous Integration

- *(update)* Update nix-cache-action `v1` -> `v2`
- *(uncategorized)* Split actionlint into its own module
- *(uncategorized)* Add darwin
- *(uncategorized)* Init gh-cache

### Flake.lock

- *(uncategorized)* Update

### Rm

- *(uncategorized)* Simple ls

### Update

- *(cargo)* `Cargo.lock`

## [0.1.2] - 2023-06-23

### Features

- *(uncategorized)* Improve handling performance
- *(uncategorized)* Inline most methods

### Bug Fixes

- *(uncategorized)* Use unicodepoints, to increase portability
- *(uncategorized)* Remove duplicate line
- *(uncategorized)* Widget clears Block style

### Documentation

- *(readme)* Update demo gif

### Styling

- *(uncategorized)* Use a clearer construct
- *(uncategorized)* Add must_use to public api
- *(uncategorized)* Prefer self

### Testing

- *(uncategorized)* Add modifier tests
- *(uncategorized)* Add cursor style tests
- *(tests)* Test terminal dimensions and styles

### Miscellaneous Tasks

- *(lint)* Add more lint targets

### Examples

- *(uncategorized)* Align `simple_ls*` highlighting differences
- *(examples)* Improve simple example
- *(examples)* Add default output from examples



## [0.1.1] - 2023-06-14

### Features

- *(bench)* Init iai
- *(style)* Allow for custom cursor styling and shapes
- *(uncategorized)* Add boundary checks for the enclosed area
- *(dependencies)* Reexport vt100
- *(style)* Add basic styling support

### Documentation

- *(demo)* Add a demo overview

### Continuous Integration

- *(init)* Typos configuration

### Example

- *(uncategorized)* Add tracing support
- *(example)* Improve input handling in smux

## [0.1.0] - 2023-06-08

- Initial release


<!-- generated by git-cliff -->
