use std::{
    io::{self, BufWriter, Read, Write},
    sync::{Arc, RwLock},
    time::Duration,
};

use bytes::Bytes;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use ratatui::{backend::CrosstermBackend, layout::Rect, style::Color, Terminal};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};
use tokio::{
    sync::mpsc::{channel, Sender},
    task::spawn_blocking,
};
use tui_term::widget::PseudoTerm;

#[derive(Debug, Clone)]
struct Size {
    cols: u16,
    rows: u16,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let (mut terminal, mut size) = setup_terminal().unwrap();

    let cwd = std::env::current_dir().unwrap();
    let shell = std::env::var("SHELL").unwrap();
    let mut cmd = CommandBuilder::new(shell);
    cmd.cwd(cwd);

    let mut panes: Vec<PtyPane> = Vec::new();
    let mut active_pane: Option<usize> = None;

    // Add a default pane
    open_new_pane(&mut panes, &mut active_pane, &cmd, size.clone())?;

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
                let pseudo_term = PseudoTerm::new(screen).block(block);
                let pane_chunk = Rect {
                    x: chunks[0].x,
                    y: chunks[0].y + (index as u16 * pane_height), // Adjust the y coordinate for each pane
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
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        cleanup_terminal(&mut terminal).unwrap();
                        return Ok(());
                    }
                    KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        open_new_pane(&mut panes, &mut active_pane, &cmd, size.clone())?;
                    }
                    KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        close_active_pane(&mut panes, &mut active_pane).await?;
                    }
                    KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if let Some(pane) = active_pane {
                            active_pane = Some(pane.saturating_sub(1));
                        }
                    }
                    KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
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
                Event::Resize(rows, cols) => {
                    for pane in panes.iter_mut() {
                        pane.parser.write().unwrap().set_size(rows, cols);
                    }
                    size.rows = rows;
                    size.cols = cols;
                }
                _ => {}
            }
        }
    }
}

struct PtyPane {
    parser: Arc<RwLock<vt100::Parser>>,
    sender: Sender<Bytes>,
}

impl PtyPane {
    fn new(size: Size, cmd: CommandBuilder) -> io::Result<Self> {
        let pty_system = native_pty_system();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: size.rows,
                cols: size.cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        let parser = Arc::new(RwLock::new(vt100::Parser::new(size.rows, size.cols, 0)));

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

        Ok(Self { parser, sender: tx })
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
