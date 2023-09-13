use std::{
    sync::{
        mpsc::{channel, Sender},
        Arc, RwLock,
    },
    thread::{self, JoinHandle},
};

use bytes::Bytes;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use portable_pty::{CommandBuilder, ExitStatus, NativePtySystem, PtyPair, PtySize, PtySystem};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Clear, StatefulWidget, Widget},
};
use vt100::{Parser, Screen};

use crate::state;

/// A widget representing a pseudo-terminal screen.
///
/// The `PseudoTerminal` widget displays the contents of a pseudo-terminal screen,
/// which is typically populated with text and control sequences from a terminal emulator.
/// It provides a visual representation of the terminal output within a TUI application.
///
/// The contents of the pseudo-terminal screen are represented by a `vt100::Screen` object.
/// The `vt100` library provides functionality for parsing and processing terminal control sequences
/// and handling terminal state, allowing the `PseudoTerminal` widget to accurately render the
/// terminal output.
///
/// # Examples
///
/// ```rust
/// use ratatui::{
///     style::{Color, Modifier, Style},
///     widgets::{Block, Borders},
/// };
/// use tui_term::widget::PseudoTerminal;
/// use vt100::Parser;
///
/// let mut parser = vt100::Parser::new(24, 80, 0);
/// let pseudo_term = PseudoTerminal::new(&parser.screen())
///     .block(Block::default().title("Terminal").borders(Borders::ALL))
///     .style(
///         Style::default()
///             .fg(Color::White)
///             .bg(Color::Black)
///             .add_modifier(Modifier::BOLD),
///     );
/// ```
#[derive(Default)]
pub struct PseudoTerminal<'a> {
    screen: Option<&'a Screen>,
    pub(crate) block: Option<Block<'a>>,
    style: Option<Style>,
    pub(crate) cursor: Cursor,
}

pub struct Cursor {
    pub(crate) symbol: String,
    pub(crate) style: Style,
    pub(crate) overlay_style: Style,
}

impl Cursor {
    /// Sets the symbol for the cursor.
    ///
    /// # Arguments
    ///
    /// * `symbol`: The symbol to set as the cursor.
    ///
    /// # Example
    ///
    /// ```
    /// use ratatui::style::Style;
    /// use tui_term::widget::Cursor;
    ///
    /// let cursor = Cursor::default().symbol("|");
    /// ```
    #[inline]
    #[must_use]
    pub fn symbol(mut self, symbol: &str) -> Self {
        self.symbol = symbol.into();
        self
    }

    /// Sets the style for the cursor.
    ///
    /// # Arguments
    ///
    /// * `style`: The `Style` to set for the cursor.
    ///
    /// # Example
    ///
    /// ```
    /// use ratatui::style::Style;
    /// use tui_term::widget::Cursor;
    ///
    /// let cursor = Cursor::default().style(Style::default());
    /// ```
    #[inline]
    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the overlay style for the cursor.
    ///
    /// The overlay style is used when the cursor overlaps with existing content on the screen.
    ///
    /// # Arguments
    ///
    /// * `overlay_style`: The `Style` to set as the overlay style for the cursor.
    ///
    /// # Example
    ///
    /// ```
    /// use ratatui::style::Style;
    /// use tui_term::widget::Cursor;
    ///
    /// let cursor = Cursor::default().overlay_style(Style::default());
    /// ```
    #[inline]
    #[must_use]
    pub const fn overlay_style(mut self, overlay_style: Style) -> Self {
        self.overlay_style = overlay_style;
        self
    }
}

impl Default for Cursor {
    #[inline]
    fn default() -> Self {
        Self {
            symbol: "\u{2588}".into(), //"█".
            style: Style::default().fg(Color::Gray),
            overlay_style: Style::default().add_modifier(Modifier::REVERSED),
        }
    }
}

impl<'a> PseudoTerminal<'a> {
    /// Creates a new instance of `PseudoTerminal`.
    ///
    /// # Arguments
    ///
    /// * `screen`: The reference to the `Screen`.
    ///
    /// # Example
    ///
    /// ```
    /// use tui_term::widget::PseudoTerminal;
    /// use vt100::Parser;
    ///
    /// let mut parser = vt100::Parser::new(24, 80, 0);
    /// let pseudo_term = PseudoTerminal::new(&parser.screen());
    /// ```
    #[inline]
    #[must_use]
    pub fn new(screen: &'a Screen) -> Self {
        PseudoTerminal {
            screen: Some(screen),
            block: None,
            style: None,
            cursor: Cursor::default(),
        }
    }

    /// Sets the block for the `PseudoTerminal`.
    ///
    /// # Arguments
    ///
    /// * `block`: The `Block` to set.
    ///
    /// # Example
    ///
    /// ```
    /// use ratatui::widgets::Block;
    /// use tui_term::widget::PseudoTerminal;
    /// use vt100::Parser;
    ///
    /// let mut parser = vt100::Parser::new(24, 80, 0);
    /// let block = Block::default();
    /// let pseudo_term = PseudoTerminal::new(&parser.screen()).block(block);
    /// ```
    #[inline]
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Sets the cursor configuration for the `PseudoTerminal`.
    ///
    /// The `cursor` method allows configuring the appearance of the cursor within the
    /// `PseudoTerminal` widget.
    ///
    /// # Arguments
    ///
    /// * `cursor`: The `Cursor` configuration to set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ratatui::style::Style;
    /// use tui_term::widget::{Cursor, PseudoTerminal};
    ///
    /// let mut parser = vt100::Parser::new(24, 80, 0);
    /// let cursor = Cursor::default().symbol("|").style(Style::default());
    /// let pseudo_term = PseudoTerminal::new(&parser.screen()).cursor(cursor);
    /// ```
    #[inline]
    #[must_use]
    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = cursor;
        self
    }

    /// Sets the style for `PseudoTerminal`.
    ///
    /// # Arguments
    ///
    /// * `style`: The `Style` to set.
    ///
    /// # Example
    ///
    /// ```
    /// use ratatui::style::Style;
    /// use tui_term::widget::PseudoTerminal;
    ///
    /// let mut parser = vt100::Parser::new(24, 80, 0);
    /// let style = Style::default();
    /// let pseudo_term = PseudoTerminal::new(&parser.screen()).style(style);
    /// ```
    #[inline]
    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    #[inline]
    #[must_use]
    pub const fn screen(&self) -> Option<&Screen> {
        self.screen
    }
}

impl Widget for PseudoTerminal<'_> {
    #[inline]
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let inner_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        state::handle(&self, inner_area, buf, None);
    }
}

pub struct PseudoTerminalState {
    pub parser: Arc<RwLock<Parser>>,
    pub pty: PtyPair,
}

// TODO: Make `handled` an enum with three variants, with unhandled events being classified as
// either ignored or explicitly skipped
pub struct EventHandlerResult {
    pub event: Event,
    pub handled: bool,
}

impl EventHandlerResult {
    fn new(event: Event, handled: bool) -> Self {
        Self { event, handled }
    }
}

impl PseudoTerminalState {
    /// Initializes a PTY and a parser, using a given initial size
    pub fn new(initial_size: Rect) -> Self {
        let (rows, cols) = (initial_size.height, initial_size.width);
        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        Self {
            parser: Arc::new(RwLock::new(Parser::new(rows, cols, 0))),
            pty: NativePtySystem::default().openpty(size).unwrap(),
        }
    }

    /// Updates the area of the PTY and the parser on a frame update
    pub fn set_area(&mut self, new_area: Rect) {
        let (rows, cols) = (new_area.height, new_area.width);

        self.pty
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();

        self.parser.write().unwrap().set_size(rows, cols);
    }

    /// Runs the given command in a separate thread
    pub fn spawn_child_process_thread(&self, command: CommandBuilder) -> JoinHandle<ExitStatus> {
        let mut child = self.pty.slave.spawn_command(command).unwrap();
        thread::spawn(move || -> ExitStatus { child.wait().unwrap() })
    }

    /// Spawns the thread which parses process output in order for it to be properly displayed
    pub fn spawn_parser_thread(&self) -> JoinHandle<()> {
        let mut reader = self.pty.master.try_clone_reader().unwrap();
        let parser = self.parser.clone();

        thread::spawn(move || {
            let mut buf = [0u8; 8192];
            let mut processed_buf = Vec::new();
            loop {
                let size = reader.read(&mut buf).unwrap();
                if size == 0 {
                    break;
                }
                if size > 0 {
                    processed_buf.extend_from_slice(&buf[..size]);
                    let mut parser = parser.write().unwrap();
                    parser.process(&processed_buf);

                    processed_buf.clear();
                }
            }
        })
    }

    /// Spawns the thread which sends user input to the process
    pub fn spawn_input_thread(&self) -> (Sender<Bytes>, JoinHandle<()>) {
        let (input_sender, input_receiver) = channel::<Bytes>();
        let mut writer = self.pty.master.take_writer().unwrap();

        (
            input_sender,
            thread::spawn(move || {
                while let Ok(bytes) = input_receiver.recv() {
                    writer.write_all(&bytes).unwrap();
                }
            }),
        )
    }

    /// Handles input from the user, sending it to the process
    /// Ignores user-specified events, usually so they can be handled by the application
    // ? Should this just return bytes which the user can choose to send/not send?
    pub fn handle_input(
        &self,
        excluded_events: &[Event],
        input_sender: &Sender<Bytes>,
    ) -> EventHandlerResult {
        // Event read is blocking
        let event = event::read().unwrap();
        if excluded_events.contains(&event) {
            return EventHandlerResult::new(event, false);
        }

        // This block avoids each arm needing to return `true` explicitly
        let handled = 'handled: {
            match event {
                Event::FocusGained
                | Event::FocusLost
                | Event::Mouse(_)
                | Event::Paste(_)
                // Resize events do not need to be handled because the frame will be
                // resized on every render call anyway
                | Event::Resize(_, _) => false,
                Event::Key(key_event) => {
                    let send = |sender: &Sender<Bytes>, bytes: &[u8]| {
                        sender.send(Bytes::from(bytes.to_vec())).unwrap();
                    };

                    if key_event.kind != KeyEventKind::Press {
                        break 'handled false;
                    }

                    match key_event.code {
                        KeyCode::Char(c) => send(input_sender, c.to_string().as_bytes()),
                        KeyCode::Backspace => send(input_sender, &[8]),
                        KeyCode::Enter => send(input_sender, &[b'\n']),
                        KeyCode::Left => send(input_sender, b"\x1b[D"),
                        KeyCode::Right => send(input_sender, b"\x1b[C"),
                        KeyCode::Up => send(input_sender, b"\x1b[A"),
                        KeyCode::Down => send(input_sender, b"\x1b[B"),
                        KeyCode::Home => send(input_sender, b"\x1b[H"),
                        KeyCode::End => send(input_sender, b"\x1b[F"),
                        KeyCode::PageUp => send(input_sender, b"\x1b[5~"),
                        KeyCode::PageDown => send(input_sender, b"\x1b[6~"),
                        KeyCode::Tab => send(input_sender, b"\t"),
                        KeyCode::BackTab => send(input_sender, b"\x1b[Z"),
                        KeyCode::Delete => send(input_sender, b"\x1b[3~"),
                        KeyCode::Insert => send(input_sender, b"\x1b[2~"),
                        _ => break 'handled false,
                    }

                    true
                }
            }
        };

        EventHandlerResult::new(event, handled)
    }
}

impl StatefulWidget for PseudoTerminal<'_> {
    type State = PseudoTerminalState;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Clear.render(area, buf);
        // Get the area inside the block borders, if there is a block
        // If there is no block, use the whole area
        let inner_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        state.set_area(inner_area);

        state::handle(&self, inner_area, buf, Some(state));
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{backend::TestBackend, widgets::Borders, Terminal};

    use super::*;

    fn snapshot_typescript(stream: &[u8]) -> String {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(24, 80, 0);
        parser.process(stream);
        let pseudo_term = PseudoTerminal::new(parser.screen());
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.size());
            })
            .unwrap();
        format!("{:?}", terminal.backend().buffer())
    }

    #[test]
    fn empty_actions() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(24, 80, 0);
        parser.process(b" ");
        let pseudo_term = PseudoTerminal::new(parser.screen());
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.size());
            })
            .unwrap();
        let view = format!("{:?}", terminal.backend().buffer());
        insta::assert_snapshot!(view);
    }
    #[test]
    fn boundary_rows_overshot_no_panic() {
        let stream = include_bytes!("../test/typescript/simple_ls.typescript");
        // Make the backend on purpose much smaller
        let backend = TestBackend::new(80, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(24, 80, 0);
        parser.process(stream);
        let pseudo_term = PseudoTerminal::new(parser.screen());
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.size());
            })
            .unwrap();
        let view = format!("{:?}", terminal.backend().buffer());
        insta::assert_snapshot!(view);
    }

    #[test]
    fn simple_ls() {
        let stream = include_bytes!("../test/typescript/simple_ls.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn simple_cursor_alternate_symbol() {
        let stream = include_bytes!("../test/typescript/simple_ls.typescript");
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(24, 80, 0);
        let cursor = Cursor::default().symbol("|");
        parser.process(stream);
        let pseudo_term = PseudoTerminal::new(parser.screen()).cursor(cursor);
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.size());
            })
            .unwrap();
        let view = format!("{:?}", terminal.backend().buffer());
        insta::assert_snapshot!(view);
    }
    #[test]
    fn simple_cursor_styled() {
        let stream = include_bytes!("../test/typescript/simple_ls.typescript");
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(24, 80, 0);
        let style = Style::default().bg(Color::Cyan).fg(Color::LightRed);
        let cursor = Cursor::default().symbol("|").style(style);
        parser.process(stream);
        let pseudo_term = PseudoTerminal::new(parser.screen()).cursor(cursor);
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.size());
            })
            .unwrap();
        let view = format!("{:?}", terminal.backend().buffer());
        insta::assert_snapshot!(view);
    }
    #[test]
    fn overlapping_cursor() {
        let stream = include_bytes!("../test/typescript/overlapping_cursor.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn overlapping_cursor_alternate_style() {
        let stream = include_bytes!("../test/typescript/overlapping_cursor.typescript");
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(24, 80, 0);
        let style = Style::default().bg(Color::Cyan).fg(Color::LightRed);
        let cursor = Cursor::default().overlay_style(style);
        parser.process(stream);
        let pseudo_term = PseudoTerminal::new(parser.screen()).cursor(cursor);
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.size());
            })
            .unwrap();
        let view = format!("{:?}", terminal.backend().buffer());
        insta::assert_snapshot!(view);
    }
    #[test]
    fn simple_ls_with_block() {
        let stream = include_bytes!("../test/typescript/simple_ls.typescript");
        let backend = TestBackend::new(100, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(24, 80, 0);
        parser.process(stream);
        let block = Block::default().borders(Borders::ALL).title("ls");
        let pseudo_term = PseudoTerminal::new(parser.screen()).block(block);
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.size());
            })
            .unwrap();
        let view = format!("{:?}", terminal.backend().buffer());
        insta::assert_snapshot!(view);
    }
    #[test]
    fn simple_ls_no_style_from_block() {
        let stream = include_bytes!("../test/typescript/simple_ls.typescript");
        let backend = TestBackend::new(100, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(24, 80, 0);
        parser.process(stream);
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .title("ls");
        let pseudo_term = PseudoTerminal::new(parser.screen()).block(block);
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.size());
            })
            .unwrap();
        let view = format!("{:?}", terminal.backend().buffer());
        insta::assert_snapshot!(view);
    }
    #[test]
    fn italic_text() {
        let stream = b"[3mThis line will be displayed in italic.[0m This should have no style.";
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn underlined_text() {
        let stream =
            b"[4mThis line will be displayed with an underline.[0m This should have no style.";
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn bold_text() {
        let stream = b"[1mThis line will be displayed bold.[0m This should have no style.";
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn inverse_text() {
        let stream = b"[7mThis line will be displayed inversed.[0m This should have no style.";
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn combined_modifier_text() {
        let stream =
            b"[4m[3mThis line will be displayed in italic and underlined.[0m This should have no style.";
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_02_01() {
        let stream = include_bytes!("../test/typescript/vttest_02_01.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_02() {
        let stream = include_bytes!("../test/typescript/vttest_02_02.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_03() {
        let stream = include_bytes!("../test/typescript/vttest_02_03.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_04() {
        let stream = include_bytes!("../test/typescript/vttest_02_04.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_05() {
        let stream = include_bytes!("../test/typescript/vttest_02_05.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_06() {
        let stream = include_bytes!("../test/typescript/vttest_02_06.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_07() {
        let stream = include_bytes!("../test/typescript/vttest_02_07.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_08() {
        let stream = include_bytes!("../test/typescript/vttest_02_08.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_09() {
        let stream = include_bytes!("../test/typescript/vttest_02_09.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_10() {
        let stream = include_bytes!("../test/typescript/vttest_02_10.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_11() {
        let stream = include_bytes!("../test/typescript/vttest_02_11.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_12() {
        let stream = include_bytes!("../test/typescript/vttest_02_12.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_13() {
        let stream = include_bytes!("../test/typescript/vttest_02_13.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_14() {
        let stream = include_bytes!("../test/typescript/vttest_02_14.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_15() {
        let stream = include_bytes!("../test/typescript/vttest_02_15.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
}
