use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode},
    execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, size},
};
use std::io::{self, stdout};

struct Cursor {
    x: u16,
    y: u16,
}

enum Mode {
    Normal,
    Insert,
}

struct Editor {
    cursor: Cursor,
    mode: Mode,
}

fn main() -> io::Result<()> {
    let mut editor = Editor {
        cursor: Cursor { x: 0, y: 0 },
        mode: Mode::Normal,
    };

    execute!(stdout(), Clear(ClearType::All))?;

    enable_raw_mode()?;

    loop {
        let (_, rows) = size()?;

        let mode_text = match editor.mode {
            Mode::Normal => "-- NORMAL --",
            Mode::Insert => "-- INSERT --",
        };

        execute!(stdout(), MoveTo(0, rows - 1), Clear(ClearType::CurrentLine),)?;

        print!("{}", mode_text);
        execute!(stdout(), MoveTo(editor.cursor.x, editor.cursor.y))?;

        if let Event::Key(key) = event::read()? {
            match editor.mode {
                Mode::Normal => match key.code {
                    KeyCode::Char('h') | KeyCode::Left => {
                        if editor.cursor.x > 0 {
                            editor.cursor.x -= 1;
                        }
                    }

                    KeyCode::Char('j') | KeyCode::Down => editor.cursor.y += 1,

                    KeyCode::Char('k') | KeyCode::Up => {
                        if editor.cursor.y > 0 {
                            editor.cursor.y -= 1;
                        }
                    }

                    KeyCode::Char('l') | KeyCode::Right => editor.cursor.x += 1,

                    KeyCode::Char('i') => editor.mode = Mode::Insert,

                    KeyCode::Char('q') => break,

                    _ => {}
                },

                Mode::Insert => match key.code {
                    KeyCode::Left => {
                        if editor.cursor.x > 0 {
                            editor.cursor.x -= 1;
                        }
                    }

                    KeyCode::Down => editor.cursor.y += 1,

                    KeyCode::Up => {
                        if editor.cursor.y > 0 {
                            editor.cursor.y -= 1;
                        }
                    }

                    KeyCode::Right => editor.cursor.x += 1,

                    KeyCode::Esc => editor.mode = Mode::Normal,

                    _ => {}
                },
            }
        }
    }

    disable_raw_mode()?;

    Ok(())
}
