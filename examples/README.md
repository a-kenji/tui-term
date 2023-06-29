# Tui-term Examples

In this folder you find examples for using `tui-term`.

To run the `simple_ls_rw` example:

```
cargo run --example simple_rw
```

** Note: The examples provided here are simplified and may omit proper error handling and edge cases for brevity and clarity. They are intended to demonstrate specific features and concepts related to `tui-term`. **

## `simple_ls_chan`

- Required: `ls`

Uses a channel to share data.
It demonstrates how to send messages from one thread to another to update the `PseudoTerm` widget.

## `simple_ls_rw`

- Required: `ls`

Uses a `RWLock` to manage shared read/write access.
The RWLock ensures that multiple threads can read from the pseudoterminal simultaneously, while exclusive write access is granted to only one thread at a time.

## `nested_shell`

- Required: `SHELL` environment variable
- Description: Demonstrates nested shell functionality.
- Uses a RWLock to manage shared read/write access.

## `nested_shell_async`

- Required: SHELL environment variable
- Description: Demonstrates nested shell functionality with asynchronous I/O using Tokio.
- Uses an RWLock to manage shared read/write access.

## `long_running`

- Required: `top` command
- Description: Displays the output of the `top` command, which makes usage of the alternate screen.

## `smux`

- Required: SHELL environment variable
- Description: This example demonstrates a simple terminal multiplexer.
- Uses: asynchronous I/O using Tokio
