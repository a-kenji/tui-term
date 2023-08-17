#![allow(unused_variables)]

use std::{io, sync::mpsc::Sender};

use bytes::Bytes;
use crossterm::{
    event::{DisableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
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
    let excluded_events = [Event::Key(KeyEvent::new(
        KeyCode::Char('c'),
        KeyModifiers::CONTROL,
    ))];
    loop {
        let input_result = state.handle_input(&excluded_events, &sender);
        if !input_result.handled {
            match input_result.event {
                Event::Key(ke)
                    if ke == KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL) =>
                {
                    return Ok(());
                }
                _ => {}
            }
        } else {
            terminal.draw(|f| ui(f, &mut state))?;
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

    let explanation = "Press Ctrl+C to exit".to_string();
    let explanation = Paragraph::new(explanation)
        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED))
        .alignment(Alignment::Center);

    f.render_stateful_widget(pseudo_term, chunks[0], state);
    f.render_widget(explanation, chunks[1]);
}
