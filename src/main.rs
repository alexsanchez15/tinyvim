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
                    buffer
                        .newline(&mut stdout)
                        .expect("failed to buffer.newline()");
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
                event::KeyCode::Esc => {
                    buffer.display();
                    break;
                }
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
        self.current_col += 1;
    }
    fn pop(&mut self) -> char {
        self.lines
            .get_mut(self.current_line)
            .unwrap()
            .pop()
            .unwrap()
    }
    fn newline(&mut self, stdout: &mut Stdout) -> io::Result<()> {
        //function for moving to the next line when enter pressed.
        self.current_line += 1;
        self.current_col = 0;
        self.total_lines += 1;
        stdout.execute(cursor::MoveToNextLine(1))?;
        if self.lines.len() <= self.current_line {
            self.lines.push(String::new());
        }

        Ok(())
    }
    fn display(&self) {
        //for debugging, just write everything in the bufferf
        let mut i = 0;
        for line in self.lines.clone() {
            println!("{}: {}", i, line);
            i += 1;
        }
    }
}
