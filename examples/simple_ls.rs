use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    style::ResetColor,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use ratatui::text::Line;
use ratatui::{
    backend::Backend,
    layout::Alignment,
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::sync::mpsc::channel;
use tui_term::widget::PseudoTerm;
use vt100::Screen;

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
    let mut cmd = CommandBuilder::new("ls");
    cmd.cwd(cwd);

    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();
    let mut child = pair.slave.spawn_command(cmd).unwrap();
    drop(pair.slave);

    let (tx, rx) = channel();
    let mut reader = pair.master.try_clone_reader().unwrap();
    let mut parser = vt100::Parser::new(24, 80, 0);

    std::thread::spawn(move || {
        // Consume the output from the child
        let mut s = String::new();
        reader.read_to_string(&mut s).unwrap();
        tx.send(s).unwrap();
    });

    {
        // Drop writer on purpose
        let _writer = pair.master.take_writer().unwrap();
    }

    // Wait for the child to complete
    let child_exit_status = child.wait().unwrap();

    drop(pair.master);

    let output = rx.recv().unwrap();
    parser.process(output.as_bytes());

    run(&mut terminal, parser.screen())?;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    println!("Exit status: {child_exit_status}");
    Ok(())
}

fn run<B: Backend>(terminal: &mut Terminal<B>, screen: &Screen) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, screen))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
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
                ratatui::layout::Constraint::Percentage(50),
                ratatui::layout::Constraint::Percentage(50),
                ratatui::layout::Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());
    let title = Line::from("[ Running: ls ]");
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().add_modifier(Modifier::BOLD));
    let pseudo_term = PseudoTerm::new(screen).block(block);
    f.render_widget(pseudo_term, chunks[1]);
    let block = Block::default().borders(Borders::ALL);
    f.render_widget(block, f.size());
    let explanation = "Press q to exit";
    let explanation = Paragraph::new(explanation)
        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED))
        .alignment(Alignment::Center);
    f.render_widget(explanation, chunks[2]);
}
