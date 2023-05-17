//! `tui-term` is a library that provides pseudoterminal widget functionality for building interactive terminal applications using `ratatui`.
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
//! use ratatui::widgets::{Block, Borders};
//! use ratatui::style::{Style, Modifier, Color};
//! use tui_term::widget::PseudoTerm;
//! use vt100::Parser;
//!
//! let mut parser = vt100::Parser::new(24, 80, 0);
//! let pseudo_term = PseudoTerm::new(&parser.screen())
//!     .block(Block::default().title("Terminal").borders(Borders::ALL))
//!     .style(Style::default().fg(Color::White).bg(Color::Black).add_modifier(Modifier::BOLD));
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
//! - The `vt100` crate is currently the only supported backend for parsing terminal control sequences, but future versions may introduce support for alternative backends.

mod state;
pub mod widget;
