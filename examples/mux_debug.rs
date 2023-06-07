use std::{
    io,
    sync::{mpsc::Sender, Arc, RwLock},
    time::Duration,
};

use bytes::Bytes;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    style::ResetColor,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use ratatui::{
    backend::Backend,
    layout::Alignment,
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tui_term::widget::PseudoTerm;
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
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let pty_system = NativePtySystem::default();
    let cwd = std::env::current_dir().unwrap();
    let shell = std::env::var("SHELL").unwrap();
    let mut cmd = CommandBuilder::new(shell);
    cmd.cwd(cwd);

    let size = Size {
        cols: terminal.size().unwrap().width,
        rows: terminal.size().unwrap().height,
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
    let parser = Arc::new(RwLock::new(vt100::Parser::new(24, 80, 0)));

    {
        let parser = parser.clone();
        std::thread::spawn(move || {
            // Consume the output from the child
            // Can't read the full buffer, since that would wait for EOF
            let mut buf = [0u8; 8192];
            loop {
                let size = reader.read(&mut buf).unwrap();
                if size == 0 {
                    break;
                }
                if !buf.is_empty() {
                    let mut parser = parser.write().unwrap();
                    parser.process(&buf);
                }
            }
        });
    }

    let (tx, rx) = std::sync::mpsc::channel::<Bytes>();

    // Drop writer on purpose
    std::thread::spawn(move || {
        let mut writer = pair.master.take_writer().unwrap();
        loop {
            let bytes = rx.recv().unwrap();
            writer.write_all(&bytes).unwrap();
        }
        // TODO: drop explicitly
        // drop(pair.master);
    });

    run(&mut terminal, parser, tx)?;

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
                                sender.send(Bytes::from_static(b"127")).unwrap();
                            }
                            KeyCode::Enter => sender.send(Bytes::from('\n'.to_string())).unwrap(),
                            KeyCode::Left => todo!(),
                            KeyCode::Right => todo!(),
                            KeyCode::Up => todo!(),
                            KeyCode::Down => todo!(),
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
    f.render_widget(ratatui::widgets::Clear, f.size());
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
    let pseudo_term = PseudoTerm::new(screen).block(block);
    f.render_widget(pseudo_term, chunks[0]);
    let explanation = "Press q to exit".to_string();
    let explanation = Paragraph::new(explanation)
        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED))
        .alignment(Alignment::Center);
    f.render_widget(explanation, chunks[1]);
}
