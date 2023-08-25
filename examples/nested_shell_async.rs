use std::{
    io::{self, BufWriter, Read, Write},
    sync::{Arc, RwLock},
    time::Duration,
};

use bytes::Bytes;
use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    style::ResetColor,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::Alignment,
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use tokio::{
    sync::mpsc::{channel, Sender},
    task,
};
use tui_term::widget::PseudoTerminal;
use vt100::Screen;

#[derive(Debug)]
struct Size {
    cols: u16,
    rows: u16,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, ResetColor)?;
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let pty_system = NativePtySystem::default();
    let cwd = std::env::current_dir().unwrap();
    let shell = std::env::var("SHELL").unwrap();
    let mut cmd = CommandBuilder::new(shell);
    cmd.cwd(cwd);

    let size = Size {
        rows: terminal.size()?.height,
        cols: terminal.size()?.width,
    };

    let pair = pty_system
        .openpty(PtySize {
            rows: size.rows,
            cols: size.cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();
    // Wait for the child to complete
    task::spawn_blocking(move || {
        let mut child = pair.slave.spawn_command(cmd).unwrap();
        let _child_exit_status = child.wait().unwrap();
        drop(pair.slave);
    });

    let mut reader = pair.master.try_clone_reader().unwrap();
    let parser = Arc::new(RwLock::new(vt100::Parser::new(size.rows, size.cols, 0)));

    {
        let parser = parser.clone();
        task::spawn_blocking(move || {
            // Consume the output from the child
            // Can't read the full buffer, since that would wait for EOF
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

                    // Clear the processed portion of the buffer
                    processed_buf.clear();
                }
            }
        });
    }

    let (tx, mut rx) = channel::<Bytes>(32);

    let mut writer = BufWriter::new(pair.master.take_writer().unwrap());

    // Drop writer on purpose
    tokio::spawn(async move {
        while let Some(bytes) = rx.recv().await {
            writer.write_all(&bytes).unwrap();
            writer.flush().unwrap();
        }
        drop(pair.master);
    });

    run(&mut terminal, parser, tx).await?;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    println!("{size:?}");
    Ok(())
}

async fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    parser: Arc<RwLock<vt100::Parser>>,
    sender: Sender<Bytes>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, parser.read().unwrap().screen()))?;

        // Event read is non-blocking
        if event::poll(Duration::from_millis(10))? {
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char(input) => sender
                                .send(Bytes::from(input.to_string().into_bytes()))
                                .await
                                .unwrap(),
                            KeyCode::Backspace => {
                                sender.send(Bytes::from(vec![8])).await.unwrap();
                            }
                            KeyCode::Enter => sender.send(Bytes::from(vec![b'\n'])).await.unwrap(),
                            KeyCode::Left => {
                                sender.send(Bytes::from(vec![27, 91, 68])).await.unwrap()
                            }
                            KeyCode::Right => {
                                sender.send(Bytes::from(vec![27, 91, 67])).await.unwrap()
                            }
                            KeyCode::Up => {
                                sender.send(Bytes::from(vec![27, 91, 65])).await.unwrap()
                            }
                            KeyCode::Down => {
                                sender.send(Bytes::from(vec![27, 91, 66])).await.unwrap()
                            }
                            KeyCode::Home => todo!(),
                            KeyCode::End => todo!(),
                            KeyCode::PageUp => todo!(),
                            KeyCode::PageDown => todo!(),
                            KeyCode::Tab => todo!(),
                            KeyCode::BackTab => todo!(),
                            KeyCode::Delete => todo!(),
                            KeyCode::Insert => todo!(),
                            KeyCode::F(_) => todo!(),
                            KeyCode::Null => todo!(),
                            KeyCode::Esc => todo!(),
                            KeyCode::CapsLock => todo!(),
                            KeyCode::ScrollLock => todo!(),
                            KeyCode::NumLock => todo!(),
                            KeyCode::PrintScreen => todo!(),
                            KeyCode::Pause => todo!(),
                            KeyCode::Menu => todo!(),
                            KeyCode::KeypadBegin => todo!(),
                            KeyCode::Media(_) => todo!(),
                            KeyCode::Modifier(_) => todo!(),
                        }
                    }
                }
                Event::FocusGained => {}
                Event::FocusLost => {}
                Event::Mouse(_) => {}
                Event::Paste(_) => todo!(),
                Event::Resize(rows, cols) => {
                    parser.write().unwrap().set_size(rows, cols);
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, screen: &Screen) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .margin(1)
        .constraints(
            [
                ratatui::layout::Constraint::Percentage(100),
                ratatui::layout::Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().add_modifier(Modifier::BOLD));
    let pseudo_term = PseudoTerminal::new(screen).block(block);
    f.render_widget(pseudo_term, chunks[0]);
    let explanation = "Press q to exit".to_string();
    let explanation = Paragraph::new(explanation)
        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED))
        .alignment(Alignment::Center);
    f.render_widget(explanation, chunks[1]);
}
