use core::fmt;
mod normal_mode;
use crossterm::{
    cursor::{self, SetCursorShape},
    event, execute,
    terminal::{self, Clear, ClearType},
    ExecutableCommand,
};
use std::{env, fs};
use std::{fs::File, io::Stdout, thread::sleep};
use std::{
    io::{self, BufRead, BufReader, Write},
    time::Duration,
};
//write up a quick little comment for commit test
fn main() -> io::Result<()> {
    //set up the environemt (clear the area and stuff) and get stdout handle
    let mut stdout = io::stdout(); //get a handle to the stdout file
    stdout.execute(terminal::Clear(ClearType::All))?; //clear the stdout
    stdout.execute(cursor::Show)?;
    stdout.execute(cursor::MoveTo(0, 0))?;
    //load the buffer from file or to nothing
    let args: Vec<String> = env::args().collect();
    let mut buffer = match args.len() {
        1 => Buffer::new(None, &mut stdout)?,
        2 => Buffer::new(args.get(1).cloned(), &mut stdout)?,
        _ => Buffer::new(None, &mut stdout)?, //for now, this is how >1 args is dealt with
    };

    //setup terminal and get handle to stdout
    terminal::enable_raw_mode()?;
    let mut status_line = StatusLine::new(&mut buffer);
    //change mode function would be ideal
    //for now, this will do

    //for now, defaulting to insert mode
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
            status_line.change_status(Status::Insert, stdout, buffer)?;
            insert_mode(buffer, stdout, status_line)?;
        }
        Status::Normal => {
            status_line.change_status(Status::Normal, stdout, buffer)?;
            normal_mode::normal_mode(buffer, stdout, status_line)?;
        }
        Status::Visual => (),
    }

    status_line.write_status(stdout, buffer)?;
    Ok(())
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
                        buffer.update_cursor(stdout)?;
                    }
                }
                event::KeyCode::Right => {
                    match buffer.get_current_line() {
                        Ok(line) => {
                            if buffer.current_col != buffer.lines.get(line).unwrap().len() {
                                buffer.current_col += 1;
                            }
                        }
                        Err(_e) => {
                            buffer.current_col += 1;
                        }
                    }
                    buffer.update_cursor(stdout)?;
                }
                _ => (),
            }
        }
        buffer.update_cursor(stdout)?;
        status_line.write_status(stdout, buffer)?;
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
    filename: String,
}
impl StatusLine {
    fn new(buffer: &mut Buffer) -> StatusLine {
        let (cols, rows) = terminal::size().unwrap();
        StatusLine {
            status: Status::Normal,
            rows,
            cols,
            filename: buffer.filename.clone(),
        }
    }
    fn write_status(&mut self, stdout: &mut Stdout, buffer: &mut Buffer) -> io::Result<()> {
        let gcl = buffer.get_current_line()?.clone();
        stdout.execute(cursor::SavePosition)?;
        stdout.execute(cursor::MoveTo(0, self.rows - 2))?;
        stdout.execute(crossterm::style::SetBackgroundColor(
            crossterm::style::Color::Red,
        ))?;
        stdout.write(
            format!(
                "{} -- '{}' get_current_line: {} col: {} current_line: {} top row: {}",
                self.status,
                self.filename,
                gcl,
                buffer.current_col,
                buffer.current_line,
                buffer.top_row
            )
            .as_bytes(),
        )?;
        stdout.execute(crossterm::style::ResetColor)?;
        stdout.execute(cursor::RestorePosition)?;
        Ok(())
    }
    fn change_status(
        &mut self,
        status: Status,
        stdout: &mut Stdout,
        buffer: &mut Buffer,
    ) -> io::Result<()> {
        self.status = status;
        self.write_status(stdout, buffer)?;
        Ok(())
    }
    fn write_commands(&mut self, stdout: &mut Stdout, start_char: char) -> io::Result<String> {
        let mut command = String::new();
        //include the starting character
        //wait until user presses enter, keep a
        execute!(
            stdout,
            cursor::SavePosition,
            cursor::MoveTo(0, self.rows - 1),
            terminal::Clear(ClearType::CurrentLine)
        )?;
        command.push(start_char);
        stdout.write_all(start_char.to_string().as_bytes())?;
        stdout.flush()?;
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
                        stdout.execute(cursor::RestorePosition)?;
                        return Ok("".to_string()); //return an empty string
                    }
                    event::KeyCode::Enter => {
                        break;
                    }
                    _ => (),
                }
            }
        }
        stdout.execute(terminal::Clear(terminal::ClearType::CurrentLine))?;
        stdout.execute(cursor::RestorePosition)?;
        Ok(command)
    }
    fn write_to_status_line(&mut self, stdout: &mut Stdout, string: String) -> io::Result<()> {
        execute!(
            stdout,
            cursor::SavePosition,
            cursor::MoveTo(0, self.rows - 1)
        )?;
        stdout.write_all(string.as_bytes())?;
        stdout.execute(cursor::RestorePosition)?;
        Ok(())
        //THIS METHOD IS UNFINISHED FIXME
    }
}

struct Buffer {
    lines: Vec<String>,
    total_lines: usize,
    current_line: usize,
    current_col: usize,
    filename: String,
    top_row: i16, //holds current postion in the buffer (for scrolling)
}
impl Buffer {
    fn new(name: Option<String>, stdout: &mut Stdout) -> io::Result<Buffer> {
        let filename = name.unwrap_or("empty filename".to_string());
        let mut total_lines: usize = 0;
        let lines = match filename.as_str() {
            "empty filename" => vec![String::new()],
            _ => match fs::File::open(&filename) {
                Ok(file) => {
                    let mut buf = Vec::new();

                    stdout.execute(cursor::SavePosition)?;
                    let reader = BufReader::new(file);
                    for line in reader.lines() {
                        total_lines += 1;
                        let line_clone = line.unwrap().clone(); //avoid conflicts
                        stdout.write_all(line_clone.as_bytes())?;
                        stdout.execute(cursor::MoveToNextLine(1))?;
                        stdout.flush()?;
                        buf.push(line_clone);
                    }
                    stdout.execute(cursor::RestorePosition)?;
                    total_lines -= 1; //because its 0 based

                    buf
                }
                Err(..) => vec![String::new()], //file not found, just make an empty buffer
                                                //this is most likely just that they want to create a new file.
            },
        };

        Ok(Buffer {
            lines,
            total_lines,
            current_line: 0,
            current_col: 0,
            filename,
            top_row: 0,
        })
    }
    fn get_current_line(&mut self) -> io::Result<usize> {
        //FIXME this does not really make sense for negative numbers
        //but prob wont have any real effect
        let (_, row) = cursor::position()?;
        let real_row = (row) as isize + self.top_row as isize; //row -1 because 0 based is desired
        if real_row < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "negative line index",
            ));
        }
        Ok(real_row as usize) //will not fail because <0 cases already handled manually
    }
    fn update_cursor(&mut self, stdout: &mut Stdout) -> io::Result<()> {
        let line = self.get_current_line().unwrap();
        if self.lines.get(line).unwrap_or(&"".to_string()).len() >= self.current_col {
            stdout.execute(cursor::MoveTo(
                self.current_col as u16,
                self.current_line as u16,
            ))?;
        } else {
            stdout.execute(cursor::MoveTo(
                self.lines.get(line).unwrap_or(&"".to_string()).len() as u16,
                self.current_line as u16,
            ))?;
        }
        Ok(())
    }
    fn push(&mut self, c: char, stdout: &mut Stdout) -> io::Result<()> {
        //ensure current column is in the correct spot (can be offset by moving)
        let real_line = self.get_current_line()?;
        if self.current_col
            == self
                .lines
                .get(real_line)
                .expect("something happened getting lines.get(current_line)")
                .len()
        {
            //case where the cursor is at the end of the line
            self.lines.get_mut(real_line).unwrap().push(c);
            self.current_col += 1;
            stdout.write(c.to_string().as_bytes())?;
            Ok(())
        } else {
            //case where cursor is somewhere in the middle of a line
            let cline = self.lines.get_mut(real_line).unwrap();
            let (s1, s2) = cline.split_at(self.current_col);
            let mut new_line = s1.to_string();
            new_line.push(c);
            new_line.push_str(s2);
            *cline = new_line; //not current line's string content is the same as that in side of
                               //the new line (so genius man good job :#)
                               //clear the current line
            stdout.execute(cursor::SavePosition)?; //save that position
            stdout.execute(cursor::MoveToColumn(0))?;
            stdout.write(self.lines.get(real_line).unwrap().as_bytes())?;
            stdout.execute(cursor::RestorePosition)?;
            self.current_col += 1;
            self.update_cursor(stdout)?;
            //clear the line
            Ok(())
        }
    }
    fn pop(&mut self) -> char {
        let row = self.get_current_line().unwrap();
        self.lines.get_mut(row).unwrap().pop().unwrap()
    }
    fn newline(&mut self, stdout: &mut Stdout) -> io::Result<()> {
        //function for moving to the next line when enter pressed.
        //first thing needed: put a new line onto the lines vector
        self.lines.push(String::new());
        let mut line = self.get_current_line()?;
        //get the contents of the line that will be moved down(can be nothing)
        //^this needs to happen to avoid some conflicts
        //move everything under the current line down one
        let mut i = self.total_lines;
        while i > line {
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
        let (s1, s2) = self.lines.get_mut(line).unwrap().split_at(self.current_col);
        let s1_clone = s1.to_string().clone(); //need these two lines to avoid ownership issues
        let s2_clone = s2.to_string().clone();
        self.lines.get_mut(line).unwrap().clear();
        self.lines.get_mut(line).unwrap().push_str(&s1_clone);
        self.lines.get_mut(line + 1).unwrap().push_str(&s2_clone);
        //after everything is moved down a line, move the cursor down one.
        self.current_line += 1;
        self.total_lines += 1;
        self.current_col = 0;
        self.update_cursor(stdout)?;
        line = self.get_current_line()?; //update the real line, not always necessary

        //'move' the lines down one (just print them again after clearing)
        i = line;
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
        //i am going to refactor all of my backspace code. this may take a while but my current
        //stuff is really bad. i want this to be significantly more dynamic, all the if statements
        //essentially hardcode everything, scrolling pretty much killed my logic.
        let first_col = 0; //more readable and i might change this at some point
        let line = self.get_current_line()?;
        self.update_cursor(stdout)?;
        //there are really only 2 major cases when pressing backspace:
        //the cursor is at the start of the line eg col 1, and when the cursor is not.
        //when it is, a line must be removed and all next lines must be moved down.
        //contents on that line must be moved to the previous line, as well as the cursor itself.
        if self.current_col == first_col {
            if line == 0 {
                return Ok(()); //first spot on file, do nothing
            }
            let mut line_contents = String::from("");
            if !self.lines.get(line).unwrap().is_empty() {
                //if the line is nonempty,
                line_contents.push_str(self.lines.get(line).unwrap());
            }
            //place the line contents onto the previous line
            self.current_col = self.lines.get(line - 1).unwrap().len();
            //write the contents at the end of the previous line
            stdout.execute(cursor::MoveTo(self.current_col as u16, (line - 1) as u16))?;
            stdout.write_all(line_contents.as_bytes())?;
            self.lines
                .get_mut(line - 1)
                .unwrap()
                .push_str(&line_contents);
            stdout.execute(cursor::MoveToNextLine(1))?; //move back to origional line, work from
                                                        //there

            //remove the current line
            self.lines.remove(line);
            self.total_lines -= 1;
            //clear everything on the current line, and everything under it
            stdout.execute(terminal::Clear(ClearType::FromCursorDown))?;
            //now from current position, which is still col 0 of the last now removed
            //line, rewrite everything under
            let mut i = line.clone();
            while i < self.total_lines {
                stdout.write_all(self.lines.get(i).unwrap().as_bytes())?;
                stdout.execute(cursor::MoveToNextLine(1))?;
                i += 1;
            }
            self.current_line -= 1;

            self.update_cursor(stdout)?;
        }
        //if current col == first_col (0th col)
        else {
            //this is the case where backspacing from somewhere in a line, not the first place
            //split the string at the current column
            let cstring = self.lines.get_mut(line).unwrap();
            let (s1, s2) = cstring.split_at(self.current_col);
            let string2_clone = s2.to_string(); //.clone kills the borrow checker, this actually
                                                //clones (what a stupid .clone() method like what how does it not do a full clone mind
                                                //blowingly stupid for such an incredible language but there probably reasons to big
                                                //for my puny vim loving brain to comprehend
            let mut changed_string = s1.to_string();
            changed_string.pop(); //because the string slice.pop will do nothing to s1
            *cstring = changed_string + s2;
            //new line is constructed, now we must update visually
            //move cursor back one, delete until end of line, place back s2
            self.current_col -= 1;
            self.update_cursor(stdout)?;
            stdout.execute(Clear(ClearType::UntilNewLine))?;
            stdout.write_all(string2_clone.as_bytes())?;
        }

        Ok(())
    }
    //function for scrolling the screen down (and maybe up also?)
    fn scroll_buf(&mut self, lines_scrolled: i8, stdout: &mut Stdout) -> io::Result<()> {
        //move every liune down one
        let max_lines = self.get_term_rows()?;
        let mut i = self.top_row + lines_scrolled as i16;
        //clear the buffer and write max lines amount of lines, starting with lines line i
        //saving cursor position is unnecessary here
        stdout.execute(cursor::MoveTo(0, 0))?;
        let mut lines_printed = 0;
        while lines_printed < max_lines {
            if i < 0 {
                stdout.execute(Clear(ClearType::CurrentLine))?;
            } else {
                match self.lines.get(i as usize) {
                    Some(line) => {
                        stdout.write_all(line.as_bytes())?;
                        stdout.execute(Clear(ClearType::UntilNewLine))?;
                    }
                    None => {
                        stdout.execute(Clear(ClearType::CurrentLine))?;
                    }
                }
            }
            lines_printed += 1;
            i += 1;
            stdout.execute(cursor::MoveToNextLine(1))?;
        }
        self.update_cursor(stdout)?;
        stdout.flush()?;
        self.top_row += lines_scrolled as i16;
        Ok(())
    }
    fn get_term_rows(&self) -> io::Result<u16> {
        let (_, rows) = terminal::size()?;
        Ok(rows - 3) //status bar, bottom line, and 0 based. so -3
    }
    fn display(self) {
        //for debugging, just write everything in the bufferf
        let mut i = 0;
        for line in self.lines.clone() {
            println!("{}: {}", i, line);
            i += 1;
        }
    }
    fn write_to_file(&mut self) -> io::Result<()> {
        //below line wants a clone, not really sure why. related to ownership of filenmae?
        let name = self.filename.clone();
        let mut file = fs::File::create(name)?;
        for line in &self.lines {
            //clone is needed to not move ownership
            writeln!(file, "{}", line)?;
        }
        Ok(())
    }
}
