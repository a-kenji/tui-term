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
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use tokio::{
    sync::mpsc::{channel, Sender},
    task::spawn_blocking,
};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use tui_term::widget::PseudoTerminal;

#[derive(Debug, Clone, Copy)]
struct Size {
    cols: u16,
    rows: u16,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    init_panic_hook();
    let (mut terminal, mut size) = setup_terminal().unwrap();

    let cwd = std::env::current_dir().unwrap();
    let mut cmd = CommandBuilder::new_default_prog();
    cmd.cwd(cwd);

    let mut panes: Vec<PtyPane> = Vec::new();
    let mut active_pane: Option<usize> = None;

    // Add a default pane
    let pane_size = calc_pane_size(size, 1);
    open_new_pane(&mut panes, &mut active_pane, &cmd, pane_size)?;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(100), Constraint::Min(1)].as_ref())
                .split(f.size());

            let pane_height = if panes.is_empty() {
                chunks[0].height
            } else {
                (chunks[0].height.saturating_sub(1)) / panes.len() as u16
            };

            for (index, pane) in panes.iter().enumerate() {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().add_modifier(Modifier::BOLD));
                let block = if Some(index) == active_pane {
                    block.style(
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::LightMagenta),
                    )
                } else {
                    block
                };
                let parser = pane.parser.read().unwrap();
                let screen = parser.screen();
                let pseudo_term = PseudoTerminal::new(screen).block(block);
                let pane_chunk = Rect {
                    x: chunks[0].x,
                    y: chunks[0].y + (index as u16 * pane_height), /* Adjust the y coordinate for
                                                                    * each pane */
                    width: chunks[0].width,
                    height: pane_height, // Use the calculated pane height directly
                };
                f.render_widget(pseudo_term, pane_chunk);
            }

            let explanation =
                "Ctrl+n to open a new pane | Ctrl+x to close the active pane | Ctrl+q to quit";
            let explanation = Paragraph::new(explanation)
                .style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED))
                .alignment(Alignment::Center);
            f.render_widget(explanation, chunks[1]);
        })?;

        if event::poll(Duration::from_millis(10))? {
            tracing::info!("Terminal Size: {:?}", terminal.size());
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        cleanup_terminal(&mut terminal).unwrap();
                        return Ok(());
                    }
                    KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        let pane_size = calc_pane_size(size, panes.len() + 1);
                        tracing::info!("Opened new pane with size: {size:?}");
                        resize_all_panes(&mut panes, pane_size);
                        open_new_pane(&mut panes, &mut active_pane, &cmd, pane_size)?;
                    }
                    KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        close_active_pane(&mut panes, &mut active_pane).await?;
                        resize_all_panes(&mut panes, pane_size);
                    }
                    KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if let Some(pane) = active_pane {
                            active_pane = Some(pane.saturating_sub(1));
                        }
                    }
                    KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if let Some(pane) = active_pane {
                            if pane < panes.len() - 1 {
                                active_pane = Some(pane.saturating_add(1));
                            }
                        }
                    }
                    _ => {
                        if let Some(index) = active_pane {
                            if handle_pane_key_event(&mut panes[index], &key).await {
                                continue;
                            }
                        }
                    }
                },
                Event::Resize(cols, rows) => {
                    tracing::info!("Resized to: rows: {} cols: {}", rows, cols);
                    size.rows = rows;
                    size.cols = cols;
                    let pane_size = calc_pane_size(size, panes.len());
                    resize_all_panes(&mut panes, pane_size);
                }
                _ => {}
            }
        }
    }
}

fn calc_pane_size(mut size: Size, nr_panes: usize) -> Size {
    size.rows -= 2;
    size.rows /= nr_panes as u16;
    size
}

fn resize_all_panes(panes: &mut Vec<PtyPane>, size: Size) {
    for pane in panes.iter() {
        pane.resize(size);
    }
}

struct PtyPane {
    parser: Arc<RwLock<vt100::Parser>>,
    sender: Sender<Bytes>,
    master_pty: Box<dyn MasterPty>,
}

impl PtyPane {
    fn new(size: Size, cmd: CommandBuilder) -> io::Result<Self> {
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

        let mut writer = BufWriter::new(pty_pair.master.take_writer().unwrap());
        // writer is moved into the tokio task below
        tokio::spawn(async move {
            while let Some(bytes) = rx.recv().await {
                writer.write_all(&bytes).unwrap();
                writer.flush().unwrap();
            }
        });

        Ok(Self {
            parser,
            sender: tx,
            master_pty: pty_pair.master,
        })
    }

    fn resize(&self, size: Size) {
        self.parser
            .write()
            .unwrap()
            .set_size(size.rows - 4, size.cols - 4);
        self.master_pty
            .resize(PtySize {
                rows: size.rows - 4,
                cols: size.cols - 4,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
    }
}

async fn handle_pane_key_event(pane: &mut PtyPane, key: &KeyEvent) -> bool {
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
        #[cfg(unix)]
        KeyCode::Enter => vec![b'\n'],
        #[cfg(windows)]
        KeyCode::Enter => vec![b'\r', b'\n'],
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

    pane.sender.send(Bytes::from(input_bytes)).await.ok();
    true
}

fn open_new_pane(
    panes: &mut Vec<PtyPane>,
    active_pane: &mut Option<usize>,
    cmd: &CommandBuilder,
    size: Size,
) -> io::Result<()> {
    let new_pane = PtyPane::new(size, cmd.clone())?;
    let new_pane_index = panes.len();
    panes.push(new_pane);
    *active_pane = Some(new_pane_index);
    Ok(())
}

async fn close_active_pane(
    panes: &mut Vec<PtyPane>,
    active_pane: &mut Option<usize>,
) -> io::Result<()> {
    if let Some(active_index) = active_pane {
        let _pane = panes.remove(*active_index);
        // TODO: shutdown pane correctly
        if !panes.is_empty() {
            let remaining_panes = panes.len();
            let new_active_index = *active_index % remaining_panes;
            *active_pane = Some(new_active_index);
        }
    }
    Ok(())
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
            .field("screen", screen)
            .field("title:", &screen.title())
            .field("icon_name:", &screen.icon_name())
            .finish()
    }
}
