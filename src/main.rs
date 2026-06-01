use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io::{self, stdout};

fn main() -> io::Result<()> {
    let mut x: u16 = 0;
    let mut y: u16 = 0;

    loop {
        enable_raw_mode()?;

        execute!(stdout(), MoveTo(x, y))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('h') => {
                    if x == 0 {
                        continue;
                    } else {
                        x -= 1;
                    }
                }

                KeyCode::Char('k') => {
                    if y == 0 {
                        continue;
                    } else {
                        y -= 1;
                    }
                }

                KeyCode::Char('j') => y += 1,

                KeyCode::Char('l') => x += 1,

                KeyCode::Char('q') => break,

                _ => {}
            }
        }

        disable_raw_mode()?;
    }

    Ok(())
}
