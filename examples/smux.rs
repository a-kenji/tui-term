use std::{
    fmt, fs,
    io::{self, BufWriter, Read, Write},
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

use bytes::Bytes;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use ratatui::{
    backend::CrosstermBackend, buffer::Buffer, layout::Rect, style::Color, widgets::Widget,
    Terminal,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};
use tokio::{
    sync::mpsc::{channel, Sender},
    task::spawn_blocking,
};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use tui_term::widget::PseudoTerm;

#[derive(Debug, Clone)]
struct Size {
    cols: u16,
    rows: u16,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    init_panic_hook();
    let (mut terminal, mut size) = setup_terminal().unwrap();

    let cwd = std::env::current_dir().unwrap();
    let shell = std::env::var("SHELL").unwrap();
    let mut cmd = CommandBuilder::new(shell);
    cmd.cwd(cwd);

    // Add a default pane
    let mut pane_size = size.clone();
    pane_size.rows -= 2;

    let horizontal_layout = Layout::default().direction(Direction::Horizontal);
    let mut root_pane = PtyPane::new(pane_size, cmd.clone(), horizontal_layout).unwrap();
    root_pane.is_focused = true;

    loop {
        {
            let root_pane = root_pane.clone();
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Percentage(100), Constraint::Min(1)].as_ref())
                    .split(f.size());

                f.render_widget(root_pane, chunks[0]);

                let explanation =
                    "Ctrl+n to open a new pane | Ctrl+x to close the active pane | Ctrl+q to quit";
                let explanation = Paragraph::new(explanation)
                    .style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED))
                    .alignment(Alignment::Center);
                f.render_widget(explanation, chunks[1]);
            })?;
        }

        if event::poll(Duration::from_millis(10))? {
            tracing::info!("Terminal Size: {:?}", terminal.size());
            tracing::info!("Count: {:?}", PtyPane::count_children(&root_pane));
            tracing::info!(
                "Focused Child Id : {:#?}",
                PtyPane::focused_child(&root_pane)
            );
            tracing::info!("RootPane : {:#?}", root_pane);
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        cleanup_terminal(&mut terminal).unwrap();
                        return Ok(());
                    }
                    KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        let pane_size = size.clone();
                        tracing::info!("Opened new pane with size: {size:?}");
                        if let Some(active_pane) = root_pane.focused() {
                            active_pane.new_pane_horizontal(pane_size, cmd.clone());
                        }
                    }
                    KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        let pane_size = size.clone();
                        tracing::info!("Opened new pane with size: {size:?}");
                        if let Some(active_pane) = root_pane.focused() {
                            active_pane.new_pane_vertical(pane_size, cmd.clone());
                        }
                    }
                    KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        PtyPane::close_focus(&mut root_pane);
                    }
                    KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        let mut focused_pane_id = PtyPane::focused_child(&root_pane);
                        focused_pane_id += 1;
                        PtyPane::focus(&mut root_pane, focused_pane_id);
                    }
                    KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        let mut focused_pane_id = PtyPane::focused_child(&root_pane);
                        tracing::info!("Focused Pane Id: {:?}", focused_pane_id);
                        focused_pane_id -= 1;
                        tracing::info!("To Focus Pane Id: {:?}", focused_pane_id);
                        PtyPane::focus(&mut root_pane, focused_pane_id);
                    }
                    _ => {
                        if let Some(focused_pane) = root_pane.focused() {
                            let _ = focused_pane.handle_key_event(&key).await;
                        }
                    }
                },
                Event::Resize(rows, cols) => {
                    tracing::info!("Resized to: rows: {} cols: {}", rows, cols);
                    size.rows = rows;
                    size.cols = cols;
                }
                _ => {}
            }
            tracing::info!("After event_read:\n");
            tracing::info!("Terminal Size: {:?}", terminal.size());
            tracing::info!("Count: {:?}", PtyPane::count_children(&root_pane));
            tracing::info!("RootPane : {:#?}", root_pane);
        }
    }
}

#[derive(Clone)]
struct PtyPane {
    parser: Arc<RwLock<vt100::Parser>>,
    sender: Sender<Bytes>,
    children: Vec<PtyPane>,
    layout: Layout,
    is_focused: bool,
}

impl PtyPane {
    fn new(size: Size, cmd: CommandBuilder, layout: Layout) -> io::Result<Self> {
        let pty_system = native_pty_system();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: size.rows - 4,
                cols: size.cols - 4,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        let parser = Arc::new(RwLock::new(vt100::Parser::new(
            size.rows - 4,
            size.cols - 4,
            0,
        )));

        spawn_blocking(move || {
            let mut child = pty_pair.slave.spawn_command(cmd).unwrap();
            let _ = child.wait();
            drop(pty_pair.slave);
        });

        {
            let mut reader = pty_pair.master.try_clone_reader().unwrap();
            let parser = parser.clone();
            tokio::spawn(async move {
                let mut processed_buf = Vec::new();
                let mut buf = [0u8; 8192];

                loop {
                    let size = reader.read(&mut buf).unwrap();
                    if size == 0 {
                        break;
                    }
                    if size > 0 {
                        processed_buf.extend_from_slice(&buf[..size]);
                        let mut parser = parser.write().unwrap();
                        parser.process(&processed_buf);

                        // Clear the processed portion of the buffer
                        processed_buf.clear();
                    }
                }
            });
        }

        let (tx, mut rx) = channel::<Bytes>(32);

        {
            let mut writer = BufWriter::new(pty_pair.master.take_writer().unwrap());
            // Drop writer on purpose
            tokio::spawn(async move {
                while let Some(bytes) = rx.recv().await {
                    writer.write_all(&bytes).unwrap();
                    writer.flush().unwrap();
                }
                drop(pty_pair.master);
            });
        }

        Ok(Self {
            children: Vec::new(),
            layout,
            parser,
            sender: tx,
            is_focused: false,
        })
    }
    pub fn new_pane_vertical(&mut self, size: Size, cmd: CommandBuilder) {
        let vertical_layout = Layout::default().direction(Direction::Vertical);
        self.layout = vertical_layout;
        let mut pty_pane = Self::new(size, cmd, Layout::default()).unwrap();
        self.is_focused = false;
        pty_pane.is_focused = true;
        self.children.push(pty_pane);
    }
    pub fn new_pane_horizontal(&mut self, size: Size, cmd: CommandBuilder) {
        let horizontal_layout = Layout::default().direction(Direction::Horizontal);
        self.layout = horizontal_layout;
        let mut pty_pane = Self::new(size, cmd, Layout::default()).unwrap();
        self.is_focused = false;
        pty_pane.is_focused = true;
        self.children.push(pty_pane);
    }
    fn count_children(pane: &PtyPane) -> usize {
        let mut count = 0;

        for child in &pane.children {
            count += 1;
            count += Self::count_children(child);
        }
        count
    }
    fn close_focus(pane: &mut PtyPane) {
        if pane.is_focused {
            return;
        } else {
            pane.is_focused = true;
        }

        for (i, child) in pane.children.clone().iter().enumerate() {
            if child.is_focused {
                pane.children.remove(i);
            }
        }

        for child in pane.children.iter_mut() {
            Self::close_focus(child);
        }
    }
    fn focused_child(pane: &PtyPane) -> usize {
        let mut count = 0;

        if pane.is_focused {
            return count;
        }

        for child in &pane.children {
            if child.is_focused {
                return count + 1;
            }
            count += 1;
            count += Self::count_children(child);
        }
        count
    }

    fn focus(pane: &mut PtyPane, focus: usize) -> bool {
        let mut is_focused = false;

        if focus == 0 {
            pane.is_focused = true;
            is_focused = true;
        } else {
            pane.is_focused = false;
        }
        let mut remaining_focus = focus;
        for child in pane.children.iter_mut() {
            if Self::focus(child, remaining_focus - 1) {
                is_focused = true;
            }
            remaining_focus -= 1;
        }
        is_focused
    }
    fn focused(&mut self) -> Option<&mut PtyPane> {
        if self.is_focused {
            return Some(self);
        }

        for child in self.children.iter_mut() {
            if let Some(focused_pane) = child.focused() {
                return Some(focused_pane);
            }
        }

        None
    }
    async fn handle_key_event(&mut self, key: &KeyEvent) -> bool {
        let input_bytes = match key.code {
            KeyCode::Char(ch) => {
                let mut send = vec![ch as u8];
                if key.modifiers == KeyModifiers::CONTROL {
                    match ch {
                        'n' => {
                            // Ignore Ctrl+n within a pane
                            return true;
                        }
                        'x' => {
                            // Close the pane
                            return false;
                        }
                        'l' => {
                            send = vec![27, 91, 50, 74];
                        }

                        _ => {}
                    }
                }
                send
            }
            KeyCode::Enter => vec![b'\n'],
            KeyCode::Backspace => vec![8],
            KeyCode::Left => vec![27, 91, 68],
            KeyCode::Right => vec![27, 91, 67],
            KeyCode::Up => vec![27, 91, 65],
            KeyCode::Down => vec![27, 91, 66],
            KeyCode::Tab => vec![9],
            KeyCode::Home => vec![27, 91, 72],
            KeyCode::End => vec![27, 91, 70],
            KeyCode::PageUp => vec![27, 91, 53, 126],
            KeyCode::PageDown => vec![27, 91, 54, 126],
            KeyCode::BackTab => vec![27, 91, 90],
            KeyCode::Delete => vec![27, 91, 51, 126],
            KeyCode::Insert => vec![27, 91, 50, 126],
            KeyCode::Esc => vec![27],
            _ => return true,
        };

        self.sender.send(Bytes::from(input_bytes)).await.ok();
        true
    }
}

impl Widget for PtyPane {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().borders(Borders::ALL);
        let block = if self.is_focused {
            block.style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::LightMagenta),
            )
        } else {
            block
        };
        let parser = self.parser.read().unwrap();
        let screen = parser.screen();
        let pseudo_term = PseudoTerm::new(screen).block(block);

        if self.children.is_empty() {
            pseudo_term.render(area, buf);
        } else {
            let mut contraints = Vec::new();
            for _ in 1..=self.children.len() + 1 {
                contraints.push(Constraint::Percentage(
                    100 / (self.children.len() as u16 + 1),
                ));
            }
            let chunks = self.layout.constraints(contraints).split(area);

            pseudo_term.render(chunks[0], buf);

            for (i, child) in self.children.iter().enumerate() {
                child.clone().render(chunks[i + 1], buf);
            }
        }
    }
}

fn setup_terminal() -> io::Result<(Terminal<CrosstermBackend<BufWriter<io::Stdout>>>, Size)> {
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(BufWriter::new(stdout));
    let mut terminal = Terminal::new(backend)?;
    let initial_size = terminal.size()?;
    let size = Size {
        rows: initial_size.height,
        cols: initial_size.width,
    };
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    Ok((terminal, size))
}

fn cleanup_terminal(
    terminal: &mut Terminal<CrosstermBackend<BufWriter<io::Stdout>>>,
) -> io::Result<()> {
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    terminal.show_cursor()?;
    terminal.clear()?;
    Ok(())
}

fn init_panic_hook() {
    let log_file = Some(PathBuf::from("/tmp/tui-term/smux.log"));
    let log_file = match log_file {
        Some(path) => {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            Some(fs::File::create(path).unwrap())
        }
        None => None,
    };

    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to output path.
        .with_max_level(Level::TRACE)
        .with_writer(Mutex::new(log_file.unwrap()))
        .with_thread_ids(true)
        .with_ansi(true)
        .with_line_number(true);

    let subscriber = subscriber.finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Set the panic hook to log panic information before panicking
    std::panic::set_hook(Box::new(|panic| {
        let original_hook = std::panic::take_hook();
        tracing::error!("Panic Error: {}", panic);
        crossterm::terminal::disable_raw_mode().expect("Could not disable raw mode");
        crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)
            .expect("Could not leave the alternate screen");

        original_hook(panic);
    }));
    tracing::debug!("Set panic hook")
}

impl fmt::Debug for PtyPane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parser = self.parser.read().unwrap();
        let screen = parser.screen();

        f.debug_struct("PtyPane")
            .field("title:", &screen.title())
            .field("icon_name:", &screen.icon_name())
            .field("children:", &self.children)
            .field("is_focused:", &self.is_focused)
            .finish()
    }
}
