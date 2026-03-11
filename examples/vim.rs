use std::{
    io::{self, Read, Write},
    sync::{mpsc::Sender, Arc, RwLock},
    time::Duration,
};

use bytes::Bytes;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use ratatui::{
    layout::Alignment,
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Frame,
};
use tui_term::widget::PseudoTerminal;
use vt100::Screen;

#[derive(Debug)]
struct Size {
    cols: u16,
    rows: u16,
}

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run_app(&mut terminal);
    ratatui::restore();
    result
}

fn run_app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let size = Size {
        rows: terminal.size()?.height,
        cols: terminal.size()?.width,
    };

    let pty_system = NativePtySystem::default();
    let cwd = std::env::current_dir().unwrap();
    let mut cmd = CommandBuilder::new("vim");
    cmd.cwd(cwd);

    let pair = pty_system
        .openpty(PtySize {
            rows: size.rows - 5,
            cols: size.cols - 4,
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
    let parser = Arc::new(RwLock::new(vt100::Parser::new(size.rows - 2, size.cols, 0)));

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

    let result = run(terminal, parser, tx);
    println!("{size:?}");
    result
}

fn run(
    terminal: &mut DefaultTerminal,
    parser: Arc<RwLock<vt100::Parser>>,
    sender: Sender<Bytes>,
) -> io::Result<()> {
    let mut in_nested = false;

    loop {
        terminal.draw(|f| ui(f, parser.read().unwrap().screen(), in_nested))?;

        // Event read is blocking
        if event::poll(Duration::from_millis(10))? {
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        if in_nested {
                            match key.code {
                                KeyCode::Char(input) => {
                                    if input == 'c' && key.modifiers == event::KeyModifiers::CONTROL
                                    {
                                        in_nested = false;
                                    } else {
                                        sender
                                            .send(Bytes::from(input.to_string().into_bytes()))
                                            .unwrap()
                                    }
                                }
                                KeyCode::Enter => {
                                    sender.send(Bytes::from(vec![b'\r'])).unwrap();
                                }
                                KeyCode::Left => {
                                    sender.send(Bytes::from(vec![27, 91, 68])).unwrap()
                                }
                                KeyCode::Right => {
                                    sender.send(Bytes::from(vec![27, 91, 67])).unwrap()
                                }
                                KeyCode::Up => sender.send(Bytes::from(vec![27, 91, 65])).unwrap(),
                                KeyCode::Down => {
                                    sender.send(Bytes::from(vec![27, 91, 66])).unwrap()
                                }
                                KeyCode::Home => {
                                    sender.send(Bytes::from(vec![27, 91, 72])).unwrap()
                                }
                                KeyCode::End => sender.send(Bytes::from(vec![27, 91, 70])).unwrap(),
                                KeyCode::PageUp => {
                                    sender.send(Bytes::from(vec![27, 91, 53, 126])).unwrap()
                                }
                                KeyCode::PageDown => {
                                    sender.send(Bytes::from(vec![27, 91, 54, 126])).unwrap()
                                }
                                KeyCode::Tab => sender.send(Bytes::from(vec![9])).unwrap(),
                                KeyCode::BackTab => {
                                    sender.send(Bytes::from(vec![27, 91, 90])).unwrap()
                                }
                                KeyCode::Delete => {
                                    sender.send(Bytes::from(vec![27, 91, 51, 126])).unwrap()
                                }
                                KeyCode::Insert => {
                                    sender.send(Bytes::from(vec![27, 91, 50, 126])).unwrap()
                                }

                                KeyCode::Esc => {
                                    sender.send(Bytes::from(vec![27])).unwrap();
                                }

                                KeyCode::Backspace => {
                                    sender.send(Bytes::from(vec![0x08])).unwrap();
                                }

                                other => {
                                    eprintln!("Dont know: {other:?}");
                                    todo!()
                                }
                            }
                        }

                        match key.code {
                            KeyCode::Enter => {
                                in_nested = true;
                            }
                            KeyCode::Char('q') => break,
                            _other => {
                                // Ignored for now
                            }
                        }
                    }
                }
                Event::FocusGained => {}
                Event::FocusLost => {}
                Event::Mouse(_) => {}
                Event::Paste(_) => todo!(),
                Event::Resize(cols, rows) => {
                    parser.write().unwrap().screen_mut().set_size(rows, cols);
                }
            }
        }
    }

    Ok(())
}

fn ui(f: &mut Frame, screen: &Screen, in_nested: bool) {
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
        .split(f.area());
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().add_modifier(Modifier::BOLD));
    let pseudo_term = PseudoTerminal::new(screen).block(block);
    f.render_widget(pseudo_term, chunks[0]);
    let explanation = if in_nested {
        "Press Ctrl-c to exit from vim".to_string()
    } else {
        "Press q to exit".to_string()
    };
    let explanation = Paragraph::new(explanation)
        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED))
        .alignment(Alignment::Center);
    f.render_widget(explanation, chunks[1]);
}
