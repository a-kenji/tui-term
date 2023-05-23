use core::time;
use std::{io, thread};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    style::ResetColor,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use ratatui::widgets::{Block, Borders};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};
use std::sync::mpsc::channel;
use tui_term::widget::PseudoTerm;

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
    let mut cmd = CommandBuilder::new("lsd");
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

    // let term_area = Rect::new(0, 0, 80, 24);
    let pseudo_term = PseudoTerm::new(parser.screen());

    // let block = Block::default().borders(Borders::ALL);
    // terminal
    //     .draw(|f| {
    //         f.render_widget(block, f.size());
    //     })
    //     .unwrap();
    // terminal
    //     .draw(|f| {
    //         f.render_widget(ratatui::widgets::Clear, term_area);
    //     })
    //     .unwrap();
    terminal
        .draw(|f| {
            f.render_widget(pseudo_term, f.size());
        })
        .unwrap();

    // restore terminal
    thread::sleep(time::Duration::from_secs(4));
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    println!("Exit status: {child_exit_status}");
    println!("Area: {:?}", terminal.size());
    Ok(())
}
