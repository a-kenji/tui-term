use std::{
    io,
    sync::{mpsc::Sender, Arc, RwLock},
    time::Duration,
};

use bytes::Bytes;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
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
use tui_term::widget::PseudoTerminal;
use vt100::Screen;

#[derive(Debug)]
struct Size {
    cols: u16,
    rows: u16,
}

fn main() -> std::io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, ResetColor)?;
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let pty_system = NativePtySystem::default();
    let cwd = std::env::current_dir().unwrap();
    let mut cmd = CommandBuilder::new_default_prog();
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
    std::thread::spawn(move || {
        let mut child = pair.slave.spawn_command(cmd).unwrap();
        let _child_exit_status = child.wait().unwrap();
        drop(pair.slave);
    });

    let mut reader = pair.master.try_clone_reader().unwrap();
    let parser = Arc::new(RwLock::new(vt100::Parser::new(size.rows, size.cols, 0)));

    {
        let parser = parser.clone();
        std::thread::spawn(move || {
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

    let (tx, rx) = std::sync::mpsc::channel::<Bytes>();

    // Drop writer on purpose
    std::thread::spawn(move || {
        let mut writer = pair.master.take_writer().unwrap();
        while let Ok(bytes) = rx.recv() {
            writer.write_all(&bytes).unwrap();
        }
        drop(pair.master);
    });

    run(&mut terminal, parser, tx)?;

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    terminal.show_cursor()?;
    println!("{size:?}");
    Ok(())
}

fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    parser: Arc<RwLock<vt100::Parser>>,
    sender: Sender<Bytes>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, parser.read().unwrap().screen()))?;

        // Event read is blocking
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
                                .unwrap(),
                            KeyCode::Backspace => {
                                sender.send(Bytes::from(vec![8])).unwrap();
                            }
                            KeyCode::Enter => {
                                #[cfg(unix)]
                                sender.send(Bytes::from(vec![b'\n'])).unwrap();
                                #[cfg(windows)]
                                sender.send(Bytes::from(vec![b'\r', b'\n'])).unwrap();
                            }
                            KeyCode::Left => sender.send(Bytes::from(vec![27, 91, 68])).unwrap(),
                            KeyCode::Right => sender.send(Bytes::from(vec![27, 91, 67])).unwrap(),
                            KeyCode::Up => sender.send(Bytes::from(vec![27, 91, 65])).unwrap(),
                            KeyCode::Down => sender.send(Bytes::from(vec![27, 91, 66])).unwrap(),
                            KeyCode::Home => sender.send(Bytes::from(vec![27, 91, 72])).unwrap(),
                            KeyCode::End => sender.send(Bytes::from(vec![27, 91, 70])).unwrap(),
                            KeyCode::PageUp => {
                                sender.send(Bytes::from(vec![27, 91, 53, 126])).unwrap()
                            }
                            KeyCode::PageDown => {
                                sender.send(Bytes::from(vec![27, 91, 54, 126])).unwrap()
                            }
                            KeyCode::Tab => sender.send(Bytes::from(vec![9])).unwrap(),
                            KeyCode::BackTab => sender.send(Bytes::from(vec![27, 91, 90])).unwrap(),
                            KeyCode::Delete => {
                                sender.send(Bytes::from(vec![27, 91, 51, 126])).unwrap()
                            }
                            KeyCode::Insert => {
                                sender.send(Bytes::from(vec![27, 91, 50, 126])).unwrap()
                            }
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
                Event::Resize(cols, rows) => {
                    parser.write().unwrap().set_size(rows, cols);
                }
            }
        }
    }
}

fn ui(f: &mut Frame, screen: &Screen) {
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
