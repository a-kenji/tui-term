use std::{
    fmt, fs,
    io::{self, BufWriter, Read, Write},
    path::PathBuf,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use bytes::Bytes;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use ratatui::{
    DefaultTerminal,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};
use tokio::{
    sync::mpsc::{Sender, channel},
    task::spawn_blocking,
};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use tui_term::widget::{Cursor, PseudoTerminal};

#[derive(Debug, Clone, Copy)]
struct Size {
    cols: u16,
    rows: u16,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    init_panic_hook();
    let mut terminal = ratatui::init();
    let result = run_smux(&mut terminal).await;
    ratatui::restore();
    result
}

async fn run_smux(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut size = Size {
        rows: terminal.size()?.height,
        cols: terminal.size()?.width,
    };

    let cwd = std::env::current_dir().unwrap();
    let mut cmd = CommandBuilder::new_default_prog();
    cmd.cwd(cwd);

    let mut panes: Vec<PtyPane> = Vec::new();
    let mut active_pane: Option<usize> = None;

    let pane_size = calc_pane_size(size, 1);
    open_new_pane(&mut panes, &mut active_pane, &cmd, pane_size)?;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(100), Constraint::Min(1)].as_ref())
                .split(f.area());

            let pane_height = if panes.is_empty() {
                chunks[0].height
            } else {
                (chunks[0].height.saturating_sub(1)) / panes.len() as u16
            };

            for (index, pane) in panes.iter().enumerate() {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().add_modifier(Modifier::BOLD));
                let mut cursor = Cursor::default();
                let block = if Some(index) == active_pane {
                    block.style(
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::LightMagenta),
                    )
                } else {
                    cursor.hide();
                    block
                };
                let parser = pane.parser.read().unwrap();
                let screen = parser.screen();
                let pseudo_term = PseudoTerminal::new(screen).block(block).cursor(cursor);
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

        cleanup_exited_panes(&mut panes, &mut active_pane);

        if panes.is_empty() {
            return Ok(());
        }
    }
}

fn cleanup_exited_panes(panes: &mut Vec<PtyPane>, active_pane: &mut Option<usize>) {
    let mut i = 0;
    while i < panes.len() {
        if !panes[i].is_alive() {
            let _removed_pane = panes.remove(i);
            if let Some(active) = active_pane {
                match (*active).cmp(&i) {
                    std::cmp::Ordering::Greater => {
                        *active = active.saturating_sub(1);
                    }
                    std::cmp::Ordering::Equal => {
                        if panes.is_empty() {
                            *active_pane = None;
                        } else if i >= panes.len() {
                            *active_pane = Some(panes.len() - 1);
                        }
                    }
                    std::cmp::Ordering::Less => {}
                }
            }
        } else {
            i += 1;
        }
    }
}

fn calc_pane_size(mut size: Size, nr_panes: usize) -> Size {
    size.rows -= 2;
    size.rows /= nr_panes as u16;
    size
}

fn resize_all_panes(panes: &mut [PtyPane], size: Size) {
    for pane in panes.iter() {
        pane.resize(size);
    }
}

struct PtyPane {
    parser: Arc<RwLock<vt100::Parser>>,
    sender: Sender<Bytes>,
    master_pty: Box<dyn MasterPty>,
    exited: Arc<AtomicBool>,
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
        let exited = Arc::new(AtomicBool::new(false));

        {
            let exited_clone = exited.clone();
            spawn_blocking(move || {
                let mut child = pty_pair.slave.spawn_command(cmd).unwrap();
                let _ = child.wait();
                exited_clone.store(true, Ordering::Relaxed);
                drop(pty_pair.slave);
            });
        }

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
            exited,
        })
    }

    fn resize(&self, size: Size) {
        self.parser
            .write()
            .unwrap()
            .screen_mut()
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

    fn is_alive(&self) -> bool {
        !self.exited.load(Ordering::Relaxed)
    }
}

async fn handle_pane_key_event(pane: &mut PtyPane, key: &KeyEvent) -> bool {
    let input_bytes = match key.code {
        KeyCode::Char(ch) => {
            let mut send = vec![ch as u8];
            let upper = ch.to_ascii_uppercase();
            if key.modifiers == KeyModifiers::CONTROL {
                match upper {
                    'N' => {
                        // Ignore Ctrl+n within a pane
                        return true;
                    }
                    'X' => {
                        // Close the pane
                        return false;
                    }
                    // https://github.com/fyne-io/terminal/blob/master/input.go
                    // https://gist.github.com/ConnerWill/d4b6c776b509add763e17f9f113fd25b
                    '2' | '@' | ' ' => send = vec![0],
                    '3' | '[' => send = vec![27],
                    '4' | '\\' => send = vec![28],
                    '5' | ']' => send = vec![29],
                    '6' | '^' => send = vec![30],
                    '7' | '-' | '_' => send = vec![31],
                    char if ('A'..='_').contains(&char) => {
                        // Since A == 65,
                        // we can safely subtract 64 to get
                        // the corresponding control character
                        let ascii_val = char as u8;
                        let ascii_to_send = ascii_val - 64;
                        send = vec![ascii_to_send];
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
        ratatui::restore();

        original_hook(panic);
    }));
    tracing::debug!("Set panic hook")
}

impl fmt::Debug for PtyPane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parser = self.parser.read().unwrap();
        let screen = parser.screen();

        f.debug_struct("PtyPane").field("screen", screen).finish()
    }
}
