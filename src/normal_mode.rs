//normal mode
use crate::change_mode;
use crate::Buffer;
use crate::{Status, StatusLine};
use crossterm::{cursor, event, terminal};
use std::fmt::Display;
use std::io::Result;
use std::io::Stdout;
use std::process::exit;
use std::str::FromStr;
pub fn normal_mode(
    buffer: &mut Buffer,
    mut stdout: &mut Stdout,
    status_line: &mut StatusLine,
) -> Result<()> {
    //make sure the status is set to normal, and its written
    status_line.write_status(stdout, buffer)?;
    //^ above lines may become obsolete i am thinking about how to handle this better

    loop {
        if let event::Event::Key(key) = event::read()? {
            match key.code {
                event::KeyCode::Char(':') => {
                    //enter 'Ex' mode
                    let command = status_line.write_commands(stdout, ':')?;
                    process_ex_commands(&command, status_line, stdout, buffer)?;
                }
                event::KeyCode::Char('i') => {
                    change_mode(Status::Insert, buffer, stdout, status_line)?;
                    break;
                }
                event::KeyCode::Char('D') => {
                    buffer.scroll_buf(1, stdout)?;
                }
                event::KeyCode::Char('U') => {
                    buffer.scroll_buf(-1, stdout)?; //scroll up (keybind to be changed)
                }
                event::KeyCode::Char('p') => {
                    status_line
                        .write_to_status_line(stdout, format!("{}", buffer.get_current_line()?))?;
                }
                event::KeyCode::Enter => {
                    buffer
                        .newline(&mut stdout)
                        .expect("failed to buffer.newline()");
                }
                event::KeyCode::Esc => break,
                _ => (),
            }
        }
        status_line.write_status(stdout, buffer)?;
    }
    Ok(())
}
enum ExCommands {
    Write,
    Quit,
    Edit,
}

impl Display for ExCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExCommands::Write => write!(f, ":w"),
            ExCommands::Quit => write!(f, ":q"),
            ExCommands::Edit => write!(f, ":edit"),
        }
    }
}
impl FromStr for ExCommands {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            ":w" => Ok(ExCommands::Write),
            ":q" => Ok(ExCommands::Quit),
            ":edit" => Ok(ExCommands::Edit),
            _ => Err(()),
        }
    }
}
fn process_ex_commands(
    command: &str,
    status_line: &mut StatusLine,
    stdout: &mut Stdout,
    buffer: &mut Buffer,
) -> Result<()> {
    //provess the Ex commands, which are the ones that start with :
    //not even thinking about redirection at this point
    if command.is_empty() {
        return Ok(());
    }
    let args: Vec<&str> = command.split_whitespace().collect();
    //tokenize
    //first token should always be a command listed in the ex_commands, from there other tokens can
    ////be handled
    if let Some(first_token) = args.get(0) {
        match ExCommands::from_str(first_token) {
            Ok(tok) => match tok {
                ExCommands::Write => {
                    buffer.write_to_file()?;
                    status_line
                        .write_to_status_line(stdout, format!("wrote to {}", buffer.filename))?;
                }
                ExCommands::Quit => exit(0),
                ExCommands::Edit => {}
            },
            Err(_) => {}
        }
    }
    Ok(())
}
