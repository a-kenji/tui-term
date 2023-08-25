# Contributing
Thank you for considering to contribute.
You are invited to contribute new features, fixes or updates, large or small.
We are always happy to receive contributions and attempt to process them in a timely manner.

## Issues
To get an overview of what can be worked on, please take a look at the [issues](https://github.com/a-kenji/tui-term/issues?q=is%3Aissue+is%3Aopen+sort%3Aupdated-desc).

## How to get tools 
For your convenience you only need one tool to contribute to `tui-term`: `nix`.
You can drop into a development shell with:
```
nix develop
```
or use `direnv`:
```
cat .envrc && direnv allow
```

If you want to set the environment manually, the rust-toolchain version
that will be assumed is referenced inside `rust-toolchain.toml`.

## Steps
There is a lint target in the `justfile`, that can be run with:
```
just lint
```

The `rustfmt` version is referenced inside the `.rustfmt-toolchain.toml`.
The `clippy` version is referenced inside `rust-toolchain.toml`, only lints targeting that version will be merged.

## How to record `script` tests
Make sure that cols and rows are (80, 24), unless explicitly desired and annotated to be different dimensions.
```
tput lines;tput cols
```
Record your session with
```
script
```

The recorded `script` session should have the correct lines and cols specified:
```
Script started on *** [TERM="tmux-256color" TTY="/dev/pts/3" COLUMNS="80" LINES="24"]
```

## Insta
We use `insta` for snapshot testing.

Failing tests can be reviewed with:
```
cargo insta review
```

## Running Benchmarks

The benchmarks can be run with: 

```
cargo bench
```

Please ensure that your machine is in a stable state and not under heavy load when running the benchmarks for accurate and consistent results.

## Cargo.lock

Although tui-term is a library, the Cargo.lock file is included to build the examples more efficiently. 
This inclusion does not affect the library's consumers. 
If a dependency is changed, please do remember to update the lock file accordingly.


# References
- [Wez Csi References](https://wezfurlong.org/wezterm/escape-sequences.html)
- [fnky ASNI Escape Codes Gist](https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797)
- [VT100 Escape Codes](https://espterm.github.io/docs/VT100%20escape%20codes.html)
- [DEC ANSI Parser](https://vt100.net/emu/dec_ansi_parser)
- [DEC ANSI Parameters](https://vt100.net/docs/vt100-ug/chapter3.html)
