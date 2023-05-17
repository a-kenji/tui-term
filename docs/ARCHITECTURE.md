# Architecture

This document provides a high level overview of the architecture of `tui-term` and some of the choices made.

## Overview

`tui-term` is a pseudoterminal widget library built on top of the ratatui crate, which provides a framework for creating interactive terminal applications.

The general flow within tui-term follows these main steps:

- Opening a pseudoterminal (pty/tty) and obtaining reading and writing handles to the underlying process.
- Reading the output from the process and using the vt100 crate to parse the output bytes.
- Displaying the parsed output on the terminal screen within the tui-term library.
- Handling user input and writing the input bytes directly to the writer handle of the underlying process.

This flow allows for seamless interaction between the user, the terminal UI, and the underlying process, while staying flexible enough for different usecases of users of the library.

## Terminal State

Currently the parsing (backend) is handled by the excellent `vt100` crate.
However, in the future, there may be plans to introduce more general abstractions and support different terminal backends.
Because of the initial complexity we limit it to the one crate currently.

## Input
The `ratatui` crate does not enforce a specific input handling pattern.
Consequently, handling input from the user is the responsibility of the consumer of the `tui-term` library.
Consumers can integrate tui-term with their own input handling logic or leverage existing input libraries to interact with the underlying process and update the UI accordingly.

## Output

The output of the underlying process, such as the executed command or program, needs to be read by the consumer of tui-term. The consumer is responsible for obtaining the output and passing it to the `vt100` crate for processing. This separation allows for flexibility and customization in how the output is consumed and displayed within the terminal UI.

## Examples

The examples try to showcase different input and output handling strategies.
