use core::time;
use std::{
    io::{self, stdout, Write},
    thread,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand, Result,
};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use ratatui::{backend::CrosstermBackend, widgets::Widget, Terminal};
use std::sync::mpsc::channel;
use termwiz::{
    caps::Capabilities,
    terminal::{buffered::BufferedTerminal, SystemTerminal, UnixTerminal},
};
use tui_term::PseudoTerm;

fn main() -> std::io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, ResetColor)?;
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut buffered_terminal =
        BufferedTerminal::new(UnixTerminal::new(Capabilities::new_from_env().unwrap()).unwrap())
            .unwrap();

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
    let pid = child.process_id().map(|i| i as i32).unwrap_or(-1);
    drop(pair.slave);

    let (tx, rx) = channel();
    let mut reader = pair.master.try_clone_reader().unwrap();
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
    // println!("child status: {:?}", child.wait().unwrap());
    let _child_exit_status = child.wait().unwrap();

    drop(pair.master);

    let output = rx.recv().unwrap();
    // for c in output.escape_debug() {
    //     print!("{}", c);
    // }

    let mut parser = termwiz::escape::parser::Parser::new();
    let actions = parser.parse_as_vec(output.as_bytes());

    let pseudo_term = PseudoTerm::new(&actions, &buffered_terminal);
    terminal
        .draw(|f| {
            f.render_widget(pseudo_term, f.size());
        })
        .unwrap();

    // or using functions
    // std::io::stdout()
    //     .execute(SetForegroundColor(Color::Blue))?
    //     .execute(SetBackgroundColor(Color::Red))?
    //     .execute(Print("Styled text here."))?
    //     .execute(ResetColor)?;

    // restore terminal
    thread::sleep(time::Duration::from_secs(5));
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    println!();
    println!();
    println!("{:?}", actions);
    Ok(())
}
