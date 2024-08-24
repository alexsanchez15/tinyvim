//normal mode
use crate::Buffer;
use crate::{Status, StatusLine};
use crossterm::{cursor, event, terminal};
use std::io::Result;
use std::io::Stdout;
pub fn normal_mode(
    buffer: &mut Buffer,
    mut stdout: &mut Stdout,
    status_line: &mut StatusLine,
) -> Result<()> {
    //make sure the status is set to normal, and its written
    status_line.write_status(stdout)?;
    //^ above lines may become obsolete i am thinking about how to handle this better

    loop {
        if let event::Event::Key(key) = event::read()? {
            match key.code {
                event::KeyCode::Char(':') => { //enter 'Ex' mode
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
        status_line.write_status(stdout)?;
    }
    Ok(())
}
