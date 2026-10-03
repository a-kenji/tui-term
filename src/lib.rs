//! `tui-term` is a library that provides pseudoterminal widget functionality for building
//! interactive terminal applications using `ratatui`.
//!
//! # Installation
//!
//! To use the `tui-term` library, add it as a dependency in your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! tui-term = "0.2.0"
//! ```
//!
//! or use `cargo add`:
//!
//! ```test
//! cargo add tui-term
//! ```
//!
//! # Examples
//!
//! ```rust
//! use ratatui_core::style::{Color, Modifier, Style};
//! use ratatui_widgets::{block::Block, borders::Borders};
//! use tui_term::widget::PseudoTerminal;
//! use vt100::Parser;
//!
//! let mut parser = vt100::Parser::new(24, 80, 0);
//! let pseudo_term = PseudoTerminal::new(parser.screen())
//!     .block(Block::default().title("Terminal").borders(Borders::ALL))
//!     .style(
//!         Style::default()
//!             .fg(Color::White)
//!             .bg(Color::Black)
//!             .add_modifier(Modifier::BOLD),
//!     );
//! ```
//!
//! For more examples, please look at the [examples](https://github.com/a-kenji/tui-term/tree/release/examples) in the repository.
//!
//! # Features
//!
//! - Support for parsing and processing terminal control sequences via pluggable backends:
//!   the default `vt100` crate, or the `rio-vt` crate behind the `rio` feature (a full
//!   terminal engine with scrollback, selection, search, and image protocols).
//!
//! # Backends
//!
//! Both backends implement the [`widget::Screen`] trait. With `vt100` you render
//! `parser.screen()` directly; with the `rio` feature you build a [`RioScreen`] snapshot
//! from a `rio_vt::crosswords::Crosswords` and render that.

#![warn(clippy::std_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::alloc_instead_of_core)]

extern crate alloc;

mod state;
#[cfg(feature = "vt100")]
mod vt100_imp;
#[cfg(feature = "rio")]
mod rio_impl;
pub mod widget;

#[cfg(feature = "unstable")]
pub mod controller;

/// Reexport of the vt100 crate to ensure correct version compatibility
#[cfg(feature = "vt100")]
pub use vt100;

/// Reexport of the rio-vt crate to ensure correct version compatibility
#[cfg(feature = "rio")]
pub use rio_vt;

/// rio-vt backend adapters implementing [`widget::Screen`].
#[cfg(feature = "rio")]
pub use rio_impl::{RioCell, RioScreen};
