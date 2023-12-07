//! `tui-term` is a library that provides pseudoterminal widget functionality for building
//! interactive terminal applications using `ratatui`.
//!
//! # Installation
//!
//! To use the `tui-term` library, add it as a dependency in your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! tui-term = "0.1.0"
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
//! use ratatui::{
//!     style::{Color, Modifier, Style},
//!     widgets::{Block, Borders},
//! };
//! use tui_term::widget::PseudoTerminal;
//! use vt100::Parser;
//!
//! let mut parser = vt100::Parser::new(24, 80, 0);
//! let pseudo_term = PseudoTerminal::new(&parser.screen())
//!     .block(Block::default().title("Terminal").borders(Borders::ALL))
//!     .style(
//!         Style::default()
//!             .fg(Color::White)
//!             .bg(Color::Black)
//!             .add_modifier(Modifier::BOLD),
//!     );
//! ```
//!
//! For more examples, please look at the [examples](https://github.com/a-kenji/tui-term/tree/main/examples) in the repository.
//!
//! # Features
//!
//! - Support for parsing and processing terminal control sequences using the `vt100` crate.
//!
//! # Limitations
//!
//! - The `vt100` crate is currently the only supported backend for parsing terminal control
//!   sequences, but future versions may introduce support for alternative backends.

mod state;
pub mod widget;

#[cfg(feature = "unstable")]
pub mod controller;

/// Reexport of the vt100 crate to ensure correct version compatibility
pub use vt100;
