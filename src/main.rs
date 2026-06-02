use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode},
    execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use std::io::{self, stdout};

struct Cursor {
    x: u16,
    y: u16,
}

fn main() -> io::Result<()> {
    let mut cursor = Cursor { x: 0, y: 0 };

    execute!(stdout(), Clear(ClearType::All))?;

    enable_raw_mode()?;

    loop {
        execute!(stdout(), MoveTo(cursor.x, cursor.y))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('h') => {
                    if cursor.x > 0 {
                        cursor.x -= 1;
                    }
                }

                KeyCode::Char('j') => cursor.y += 1,

                KeyCode::Char('k') => {
                    if cursor.y > 0 {
                        cursor.y -= 1;
                    }
                }

                KeyCode::Char('l') => cursor.x += 1,

                KeyCode::Char('q') => break,

                _ => {}
            }
        }
    }

    disable_raw_mode()?;

    Ok(())
}
