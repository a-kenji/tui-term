//! This is an unstable interface, and can be activatet with the following
//! feature flag: `unstable`.
//!
//! The controller aims to help manage spawning and reading processes
//! to simplify the usage of `tui-term`, with the tradeoff being less flexible.
//!
//! Please do test this interface out and submit feedback, improvements and bug reports.
//!
//!
//! Currently only oneshot commands are supported by the controller:
//! Commands like `ls`, `cat`.
//! Commands like `htop`, that are persistent still need to be handled manually,
//! please look at the examples for a better overview.

use std::{
    io::Result as IoResult,
    sync::{Arc, RwLock},
};

use portable_pty::{CommandBuilder, ExitStatus, PtySystem};
use vt100::{Parser, Screen};

/// Controller, in charge of command dispatch
pub struct Controller {
    // Needs to be set
    cmd: CommandBuilder,
    size: Size,
    parser: Option<Arc<RwLock<Parser>>>,
    exit_status: Option<IoResult<ExitStatus>>,
}

impl Controller {
    pub fn new(cmd: CommandBuilder, size: Option<Size>) -> Self {
        Self {
            cmd,
            size: size.unwrap_or_default(),
            parser: None,
            exit_status: None,
        }
    }

    /// This function is blocking while waiting for the command to end.
    pub fn run(&mut self) {
        let pair = self.init_pty();
        let mut child = pair.slave.spawn_command(self.cmd.clone()).unwrap();
        drop(pair.slave);
        let mut reader = pair.master.try_clone_reader().unwrap();
        let parser = Arc::new(RwLock::new(vt100::Parser::new(
            self.size.rows,
            self.size.cols,
            0,
        )));
        {
            let parser = parser.clone();
            std::thread::spawn(move || {
                // Consume the output from the child
                let mut s = String::new();
                reader.read_to_string(&mut s).unwrap();
                if !s.is_empty() {
                    let mut parser = parser.write().unwrap();
                    parser.process(s.as_bytes());
                }
            });
        }
        // Wait for the child to complete
        self.exit_status = Some(child.wait());
        // Drop writer on purpose
        let _writer = pair.master.take_writer().unwrap();

        drop(pair.master);
        self.parser = Some(parser);
    }

    fn init_pty(&self) -> portable_pty::PtyPair {
        use portable_pty::{NativePtySystem, PtySize};
        let pty_system = NativePtySystem::default();

        pty_system
            .openpty(PtySize {
                rows: self.size.rows,
                cols: self.size.cols,
                pixel_width: self.size.pixel_width,
                pixel_height: self.size.pixel_height,
            })
            .unwrap()
    }

    pub fn screen(&self) -> Option<Screen> {
        if let Some(parser) = &self.parser {
            // We convert the read error into an option, since we might call
            // the read multiple times, but we only care that we can read at some point
            let binding = parser.read().ok()?;
            Some(binding.screen().clone())
        } else {
            None
        }
    }

    /// Whether the command finished running
    pub fn finished(&self) -> bool {
        self.exit_status.is_some()
    }

    /// The exit status of the process
    pub fn status(&self) -> Option<&IoResult<ExitStatus>> {
        self.exit_status.as_ref()
    }
}

#[derive(Default, Clone)]
pub struct Size {
    pub cols: u16,
    pub rows: u16,
    pixel_width: u16,
    pixel_height: u16,
}

impl Size {
    pub fn new(cols: u16, rows: u16, pixel_width: u16, pixel_height: u16) -> Self {
        Self {
            cols,
            rows,
            pixel_width,
            pixel_height,
        }
    }
}
