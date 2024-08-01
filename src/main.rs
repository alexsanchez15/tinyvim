use crossterm::{
    cursor, event, execute,
    terminal::{self, ClearType},
    ExecutableCommand,
};
use std::{io::Stdout, thread::sleep};
use std::{
    io::{self, Write},
    time::Duration,
};
fn main() -> io::Result<()> {
    //load the buffer from file or to nothing
    let buffer = load_buffer();

    //setup terminal and get handle to stdout
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout(); //get a handle to the stdout file
    stdout.execute(terminal::Clear(ClearType::All))?; //clear the stdout
    stdout.execute(cursor::Show)?;
    stdout.execute(cursor::MoveTo(0, 0))?;

    //for now, defaulting to insert mode
    insert_mode(buffer, stdout)?;
    Ok(())
}
fn load_buffer() -> String {
    let buf = String::new();
    return buf;
}
fn insert_mode(mut buffer: String, mut stdout: Stdout) -> io::Result<()> {
    loop {
        //refresh the buffer on screen.
        stdout.flush()?; //flush the buffer, ensureing everything is correctly placed before moving
                         //on
        if let event::Event::Key(key) = event::read()? {
            match key.code {
                event::KeyCode::Char(c) => {
                    buffer.push(c);
                    stdout.write(c.to_string().as_bytes())?;
                }
                event::KeyCode::Enter => {
                    buffer.push('\n');
                    stdout.execute(cursor::MoveToNextLine(1))?;
                }
                event::KeyCode::Backspace => {
                    let char_being_removed = buffer.pop();
                    match char_being_removed {
                        Some('\n') => {
                            stdout.execute(cursor::MoveToPreviousLine(1))?;
                        }
                        Some(_c) => {
                            //the char doesnt really matter
                            stdout.execute(cursor::MoveLeft(1))?;
                            stdout.write(" ".as_bytes())?; //clear visually
                            stdout.execute(cursor::MoveLeft(1))?; //have to do this again
                        }
                        _ => {}
                    }
                }
                event::KeyCode::Esc => break,
                _ => {}
            }
        }
    }
    println!("The contents of the buffer:\n{}", buffer);
    Ok(())
}
