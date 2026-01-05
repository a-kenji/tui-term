use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use portable_pty::CommandBuilder;
use ratatui::{
    DefaultTerminal, Frame,
    layout::Alignment,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use tui_term::{controller::Controller, widget::PseudoTerminal};
use vt100::Screen;

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run_app(&mut terminal);
    ratatui::restore();
    result
}

fn run_app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let terminal_size = terminal.size()?;
    let size = Size {
        rows: terminal_size.height,
        cols: terminal_size.width,
    };

    // Subtract the borders from the size
    let size = tui_term::controller::Size::new(size.cols - 2, size.rows, 0, 0);

    let mut cmd = CommandBuilder::new("ls");
    if let Ok(cwd) = std::env::current_dir() {
        cmd.cwd(cwd);
    }

    let mut controller = Controller::new(cmd, Some(size));
    controller.run();
    let screen = controller.screen();

    run(terminal, screen)
}

fn run(terminal: &mut DefaultTerminal, screen: Option<vt100::Screen>) -> io::Result<()> {
    loop {
        if let Some(ref screen) = screen {
            terminal.draw(|f| ui(f, &screen))?;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
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
        .split(f.area());
    let title = Line::from("[ Running: ls ]");
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().add_modifier(Modifier::BOLD));
    let pseudo_term = PseudoTerminal::new(screen)
        .cursor(tui_term::widget::Cursor::default().visibility(false))
        .block(block.clone());
    f.render_widget(pseudo_term, chunks[0]);
    let explanation = "Press q to exit";
    let explanation = Paragraph::new(explanation)
        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED))
        .alignment(Alignment::Center);
    f.render_widget(explanation, chunks[1]);
}

#[derive(Debug, Clone)]
struct Size {
    cols: u16,
    rows: u16,
}
