use std::io::{stdout, Write};

use crossterm::{
    event, execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand, Result,
};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::sync::mpsc::channel;

fn main() -> std::io::Result<()> {
    // using the macro
    execute!(
        stdout(),
        SetForegroundColor(Color::Blue),
        SetBackgroundColor(Color::Red),
        Print("Styled text here."),
        ResetColor
    )?;

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
    println!("child status: {:?}", child.wait().unwrap());

    drop(pair.master);

    let output = rx.recv().unwrap();
    for c in output.escape_debug() {
        print!("{}", c);
    }

    let mut parser = termwiz::escape::parser::Parser::new();
    let actions = parser.parse_as_vec(output.as_bytes());

    // or using functions
    stdout()
        .execute(SetForegroundColor(Color::Blue))?
        .execute(SetBackgroundColor(Color::Red))?
        .execute(Print("Styled text here."))?
        .execute(ResetColor)?;

    println!("{:?}", actions);
    Ok(())
}
