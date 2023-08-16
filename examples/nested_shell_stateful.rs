#![allow(unused_variables)]

use std::{io, sync::mpsc::Sender, time::Duration};

use bytes::Bytes;
use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    style::ResetColor,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use portable_pty::CommandBuilder;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::Alignment,
    prelude::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    terminal::Terminal,
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_term::widget::{PseudoTerminal, PseudoTerminalState};

fn main() -> std::io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, ResetColor)?;
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let cwd = std::env::current_dir().unwrap();
    let shell = std::env::var("SHELL").unwrap();
    let mut command = CommandBuilder::new(shell);
    command.cwd(cwd);

    let initial_size = terminal.size()?;
    let terminal_state = PseudoTerminalState::new(initial_size);

    let child_process_thread = terminal_state.spawn_child_process_thread(command);
    let parser_thread = terminal_state.spawn_parser_thread();
    let (input_sender, input_thread) = terminal_state.spawn_input_thread();

    run(&mut terminal, terminal_state, input_sender)?;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    mut state: PseudoTerminalState,
    sender: Sender<Bytes>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut state))?;

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
                            KeyCode::Enter => sender.send(Bytes::from(vec![b'\n'])).unwrap(),
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
                Event::Resize(rows, cols) => {
                    state.parser.write().unwrap().set_size(rows, cols);
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, state: &mut PseudoTerminalState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
        .split(f.size());

    let terminal_block = Block::default()
        .title(Span::styled("Terminal", Style::default().fg(Color::Green)))
        .borders(Borders::ALL)
        .style(Style::default().add_modifier(Modifier::BOLD));

    let pseudo_term = PseudoTerminal::default().block(terminal_block);

    let explanation = "Press q to exit".to_string();
    let explanation = Paragraph::new(explanation)
        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED))
        .alignment(Alignment::Center);

    f.render_stateful_widget(pseudo_term, chunks[0], state);
    f.render_widget(explanation, chunks[1]);
}
