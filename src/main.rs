use crossterm::{
    cursor::{self, SetCursorShape},
    event, execute,
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
    insert_mode(Buffer::new(), &mut stdout)?;
    Ok(())
}
fn load_buffer() -> String {
    let buf = String::new();
    return buf;
}

fn insert_mode(mut buffer: Buffer, mut stdout: &mut Stdout) -> io::Result<()> {
    stdout
        .execute(cursor::SetCursorShape(cursor::CursorShape::Line))
        .expect("error turning cursor into line");

    loop {
        //refresh the buffer on screen.
        stdout.flush()?; //flush the buffer, ensureing everything is correctly placed before moving
                         //on
        if let event::Event::Key(key) = event::read()? {
            match key.code {
                event::KeyCode::Char(c) => {
                    //first ensure cursor is in the right place (movement can offset it)

                    let (col, _) = cursor::position()?;
                    if buffer.current_col != col as usize {
                        buffer.current_col = col as usize;
                    }
                    //the push the character
                    buffer.push(c, &mut stdout)?;
                }
                event::KeyCode::Enter => {
                    buffer
                        .newline(&mut stdout)
                        .expect("failed to buffer.newline()");
                }
                event::KeyCode::Backspace => {
                    buffer.backspace(&mut stdout)?;
                }
                event::KeyCode::Esc => {
                    println!();
                    buffer.display();
                    break;
                }
                event::KeyCode::Up => {
                    if buffer.current_line != 0 {
                        buffer.current_line -= 1; //go up one is same as -1 to current line
                        buffer.update_cursor(&mut stdout)?;
                    }
                }
                event::KeyCode::Down => {
                    if buffer.current_line != buffer.total_lines {
                        buffer.current_line += 1;
                        buffer.update_cursor(&mut stdout)?;
                    }
                }
                event::KeyCode::Left => {
                    if buffer.current_col != 0 {
                        buffer.current_col -= 1;
                        buffer.update_cursor(&mut stdout)?;
                    }
                }
                event::KeyCode::Right => {
                    if buffer.current_col != buffer.lines.get(buffer.current_line).unwrap().len() {
                        buffer.current_col += 1;
                        buffer.update_cursor(&mut stdout)?;
                    }
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
    fn update_cursor(&mut self, stdout: &mut Stdout) -> io::Result<()> {
        if self.lines.get(self.current_line).unwrap().len() >= self.current_col {
            stdout.execute(cursor::MoveTo(
                self.current_col as u16,
                self.current_line as u16,
            ))?;
        } else {
            stdout.execute(cursor::MoveTo(
                self.lines.get(self.current_line).unwrap().len() as u16,
                self.current_line as u16,
            ))?;
        }
        Ok(())
    }
    fn push(&mut self, c: char, stdout: &mut Stdout) -> io::Result<()> {
        //ensure current column is in the correct spot (can be offset by moving)
        if self.current_col
            == self
                .lines
                .get(self.current_line)
                .expect("something happened getting lines.get(current_line)")
                .len()
        {
            //case where the cursor is at the end of the line
            self.lines.get_mut(self.current_line).unwrap().push(c);
            self.current_col += 1;
            stdout.write(c.to_string().as_bytes())?;
            Ok(())
        } else {
            //case where cursor is somewhere in the middle of a line
            let cline = self.lines.get_mut(self.current_line).unwrap();
            let (s1, s2) = cline.split_at(self.current_col);
            let mut new_line = s1.to_string();
            new_line.push(c);
            new_line.push_str(s2);
            *cline = new_line; //not current line's string content is the same as that in side of
                               //the new line (so genius man good job :#)
                               //clear the current line
            stdout.execute(cursor::SavePosition)?; //save that position
            stdout.execute(terminal::Clear(ClearType::CurrentLine))?;
            stdout.execute(cursor::MoveToColumn(0))?;
            stdout.write(self.lines.get(self.current_line).unwrap().as_bytes())?;
            stdout.execute(cursor::RestorePosition)?;
            self.current_col += 1;
            self.update_cursor(stdout)?;
            //clear the line
            Ok(())
        }
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
    fn backspace(&mut self, stdout: &mut Stdout) -> io::Result<()> {
        if cursor::position().unwrap() == (0, 0) {
            //nothing to backspace.
            return Ok(());
        }
        if self.current_col == self.lines.get(self.current_line).unwrap().len() {
            if self.current_col < 1 {
                //delete a line
                if self.total_lines <= 1 {
                    return Ok(());
                }
                self.lines.remove(self.current_line);
                self.current_line -= 1;
                self.total_lines -= 1;
                if self.lines.get(self.current_line).unwrap().len() == 0 {
                    stdout.execute(cursor::MoveToPreviousLine(1))?;
                    self.current_col = 0;
                } else {
                    self.current_col = self.lines.get(self.current_line).unwrap().len();
                    stdout.execute(cursor::MoveUp(1))?;
                    stdout.execute(cursor::MoveRight(self.current_col as u16))?;
                }
            } else {
                self.pop();
                stdout.execute(cursor::MoveLeft(1))?;
                stdout.write(" ".as_bytes())?; //clear visually
                stdout.execute(cursor::MoveLeft(1))?; //have to do this again
                self.current_col -= 1;
            }
        } else {
            //not deleting from the front of a line
            if self.current_col < 1 {
                //delete a line
                if self.total_lines <= 1 {
                    return Ok(());
                }
                //all of the contents of the line can be saved
                let contents = self.lines.get(self.current_line).unwrap().clone();

                self.lines.remove(self.current_line);
                self.current_line -= 1;
                self.total_lines -= 1;
                self.current_col = self.lines.get(self.current_line).unwrap().len();
                //clear the current line
                stdout.execute(terminal::Clear(ClearType::CurrentLine))?;
                self.update_cursor(stdout)?;
                stdout.write(&contents.as_bytes())?;
                self.lines
                    .get_mut(self.current_line)
                    .unwrap()
                    .push_str(&contents);
                self.current_col = self.lines.get(self.current_line).unwrap().len();
                self.update_cursor(stdout)?;
            } else {
                self.pop();
                stdout.execute(cursor::MoveLeft(1))?;
                stdout.write(" ".as_bytes())?; //clear visually
                stdout.execute(cursor::MoveLeft(1))?; //have to do this again
                self.current_col -= 1;
            }
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
