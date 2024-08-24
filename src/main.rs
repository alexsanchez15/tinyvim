use core::fmt;
mod normal_mode;

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
    let mut status_line = StatusLine::new();
    //change mode function would be ideal
    //for now, this will do

    //for now, defaulting to insert mode
    let mut buffer = Buffer::new();
    change_mode(Status::Insert, &mut buffer, &mut stdout, &mut status_line)?;
    Ok(())
}
fn change_mode(
    status: Status,
    buffer: &mut Buffer,
    stdout: &mut Stdout,
    status_line: &mut StatusLine,
) -> io::Result<()> {
    match status {
        Status::Insert => {
            status_line.change_status(Status::Insert);
            insert_mode(buffer, stdout, status_line)?;
        }
        Status::Normal => {
            status_line.change_status(Status::Normal);
            normal_mode::normal_mode(buffer, stdout, status_line)?;
        }
        Status::Visual => (),
    }

    status_line.write_status(stdout)?;
    Ok(())
}

fn load_buffer() -> String {
    let buf = String::new();
    return buf;
}

fn insert_mode(
    buffer: &mut Buffer,
    mut stdout: &mut Stdout,
    status_line: &mut StatusLine,
) -> io::Result<()> {
    stdout
        .execute(cursor::SetCursorShape(cursor::CursorShape::Line))
        .expect("error turning cursor into line");
    //set the status line to refelct that now in insert node FIXME situation
    status_line.change_status(Status::Insert);
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
                    break; //not really sure if this break is necessary, but keeping it to avoid
                           //annoying lines everywhere about this loop being unescapable.
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
            status_line.write_status(stdout)?;
        }
    }

    change_mode(Status::Normal, buffer, &mut stdout, status_line)?;
    Ok(())
}
enum Status {
    Normal,
    Insert,
    Visual,
}
impl fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Status::Visual => "VISUAL",
            Status::Normal => "NORMAL",
            Status::Insert => "INSERT",
        };
        write!(f, "{}", name)
    }
}
struct StatusLine {
    //enumerated things
    status: Status,
    rows: u16,
    cols: u16,
}
impl StatusLine {
    fn new() -> StatusLine {
        let (cols, rows) = terminal::size().unwrap();
        StatusLine {
            status: Status::Normal,
            rows,
            cols,
        }
    }
    fn write_status(&mut self, stdout: &mut Stdout) -> io::Result<()> {
        stdout.execute(cursor::SavePosition)?;
        stdout.execute(cursor::MoveTo(0, self.rows - 2))?;
        stdout.execute(crossterm::style::SetBackgroundColor(
            crossterm::style::Color::Red,
        ))?;
        stdout.write(
            format!(
                "{} -- 'filename' -current row: {} and column-",
                self.status, self.rows
            )
            .as_bytes(),
        )?;
        stdout.execute(crossterm::style::ResetColor)?;
        stdout.execute(cursor::RestorePosition)?;
        Ok(())
    }
    fn change_status(&mut self, status: Status) {
        self.status = status;
    }
    fn write_commands(&mut self, stdout: &mut Stdout) -> io::Result<String> {
        let mut command = String::new();
        //wait until user presses enter, keep a
        execute!(stdout, cursor::SavePosition)?;
        stdout.execute(cursor::MoveTo(0, self.rows - 1))?;
        //self.rows-1 is where this will exit
        loop {
            //basic buffer editing stuff
            if let Ok(event::Event::Key(key)) = event::read() {
                match key.code {
                    event::KeyCode::Char(c) => {
                        command.push(c);
                        stdout.write_all(c.to_string().as_bytes())?;
                        stdout.flush()?;
                    }
                    event::KeyCode::Backspace => {
                        command.pop();
                        execute!(
                            stdout,
                            cursor::MoveLeft(1),
                            terminal::Clear(terminal::ClearType::UntilNewLine)
                        )?;
                    }
                    event::KeyCode::Esc => {
                        return Ok("".to_string()); //return an empty string
                    }
                    event::KeyCode::Enter => {
                        break;
                    }
                    _ => (),
                }
            }
        }
        stdout.execute(cursor::RestorePosition)?;
        Ok(command)
    }
    fn write_to_status_line(&mut self, stdout: &mut Stdout, string: String) -> io::Result<()> {
        stdout.write_all(string.as_bytes())?;
        Ok(())
        //THIS METHOD IS UNFINISHED FIXME
    }
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
        //first thing needed: put a new line onto the lines vector
        self.lines.push(String::new());
        //get the contents of the line that will be moved down(can be nothing)

        //^this needs to happen to avoid some conflicts
        //move everything under the current line down one
        let mut i = self.total_lines;
        while i > self.current_line {
            let line_contents = self.lines.get(i).expect("l1e").to_string();
            self.lines
                .get_mut(i + 1)
                .expect("l2e")
                .push_str(&line_contents);
            self.lines.get_mut(i).unwrap().clear();
            i -= 1;
        }

        //delete everything under the cursor and have it written on the next line down
        self.update_cursor(stdout)?; //because the last thing confuses the cursor :p
        stdout.execute(terminal::Clear(ClearType::FromCursorDown))?;
        //write the text on the line down one line if its being moved (if necessary)
        let (s1, s2) = self
            .lines
            .get_mut(self.current_line)
            .unwrap()
            .split_at(self.current_col);
        let s1_clone = s1.to_string().clone(); //need these two lines to avoid ownership issues
        let s2_clone = s2.to_string().clone();
        self.lines.get_mut(self.current_line).unwrap().clear();
        self.lines
            .get_mut(self.current_line)
            .unwrap()
            .push_str(&s1_clone);
        self.lines
            .get_mut(self.current_line + 1)
            .unwrap()
            .push_str(&s2_clone);
        //after everything is moved down a line, move the cursor down one.
        self.current_line += 1;
        self.total_lines += 1;
        self.current_col = 0;
        self.update_cursor(stdout)?;

        //'move' the lines down one (just print them again after clearing)
        i = self.current_line;
        while i <= self.total_lines {
            stdout.flush()?;
            stdout.write(self.lines.get(i).unwrap().as_bytes())?;
            stdout.execute(cursor::MoveToNextLine(1))?;
            i += 1;
        }
        self.update_cursor(stdout)?;

        Ok(())
    }
    fn backspace(&mut self, stdout: &mut Stdout) -> io::Result<()> {
        self.update_cursor(stdout)?;
        if cursor::position().unwrap() == (0, 0) {
            //nothing to backspace.
            return Ok(());
        }
        if self.current_col == self.lines.get(self.current_line).unwrap().len() {
            if self.current_col < 1 {
                //delete a line
                if self.total_lines <= 0 {
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
                //split the line pop from the first line and clear the rest and then combine them
                //back is the idea
                let cline = self.lines.get_mut(self.current_line).unwrap();
                let (s1, s2) = cline.split_at(self.current_col);
                let s2_clone = s2.to_string().clone();
                let mut new_line = s1.to_string();
                new_line.pop();
                new_line.push_str(s2);
                *cline = new_line;
                //'move' the text back one
                self.current_col -= 1;
                self.update_cursor(stdout)?;
                stdout.execute(cursor::SavePosition)?;
                stdout.execute(terminal::Clear(ClearType::UntilNewLine))?;
                stdout.flush()?;
                stdout.write(s2_clone.as_bytes())?;
                stdout.execute(cursor::RestorePosition)?;
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
