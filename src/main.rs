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

    //setup terminal and get handle to stdout
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout(); //get a handle to the stdout file
    stdout.execute(terminal::Clear(ClearType::All))?; //clear the stdout
    stdout.execute(cursor::Show)?;
    stdout.execute(cursor::MoveTo(0, 0))?;

    //for now, defaulting to insert mode
    insert_mode(Buffer::new(), stdout)?;
    Ok(())
}
fn load_buffer() -> String {
    let buf = String::new();
    return buf;
}
fn insert_mode(mut buffer: Buffer, mut stdout: Stdout) -> io::Result<()> {
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
                        '\n' => {
                            stdout.execute(cursor::MoveToPreviousLine(1))?;
                        }
                        _ => {
                            //the char doesnt really matter
                            stdout.execute(cursor::MoveLeft(1))?;
                            stdout.write(" ".as_bytes())?; //clear visually
                            stdout.execute(cursor::MoveLeft(1))?; //have to do this again
                        }
                    }
                }
                event::KeyCode::Esc => break,
                _ => {}
            }
        }
    }
    Ok(())
}

struct Buffer {
    lines: Vec<String>,
    total_lines: usize,
    current_line: usize,
    current_col: usize,
}
impl Buffer {
    fn new() -> Buffer {
        Buffer {
            lines: vec![String::new()], //creates default empty line
            total_lines: 0,
            current_line: 0,
            current_col: 0,
        }
    }
    fn push(&mut self, c: char) {
        self.lines.get_mut(self.current_line).unwrap().push(c);
    }
    fn pop(&mut self) -> char {
        self.lines
            .get_mut(self.current_line)
            .unwrap()
            .pop()
            .unwrap()
    }
}
