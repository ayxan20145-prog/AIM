use crate::explorer;

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode},
    execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, size},
};
use std::io::{self, Write, stdout};

pub struct Cursor {
    x: u16,
    y: u16,
}

pub enum Mode {
    Normal,
    Insert,
}

pub struct Editor {
    cursor: Cursor,
    mode: Mode,
    lines: Vec<String>,
}

pub fn run() -> io::Result<()> {

    let mut editor = Editor {
        cursor: Cursor { x: 0, y: 0 },
        mode: Mode::Normal,
        lines: vec![String::new()],
    };

    execute!(stdout(), Clear(ClearType::All))?;

    enable_raw_mode()?;

    loop {
        let (_, rows) = size()?;

        let mode_text = match editor.mode {
            Mode::Normal => "-- NORMAL --",
            Mode::Insert => "-- INSERT --",
        };

        execute!(stdout(), Clear(ClearType::All))?;

        for (i, line) in editor.lines.iter().enumerate() {
            execute!(stdout(), MoveTo(0, i as u16))?;
            print!("{}", line);
        }

        execute!(stdout(), MoveTo(0, rows - 1), Clear(ClearType::CurrentLine),)?;

        print!("{}", mode_text);

        execute!(stdout(), MoveTo(editor.cursor.x, editor.cursor.y))?;

        stdout().flush()?;

        if let Event::Key(key) = event::read()? {
            match key.code {
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

                _ => {}
            }
            match editor.mode {
                Mode::Normal => match key.code {
                    KeyCode::Char('h') => {
                        if editor.cursor.x > 0 {
                            editor.cursor.x -= 1;
                        }
                    }

                    KeyCode::Char('j') => editor.cursor.y += 1,

                    KeyCode::Char('k') => {
                        if editor.cursor.y > 0 {
                            editor.cursor.y -= 1;
                        }
                    }

                    KeyCode::Char('l') => editor.cursor.x += 1,

                    KeyCode::Char('i') => editor.mode = Mode::Insert,

                    KeyCode::Char('e') => {
                        disable_raw_mode()?;
                        explorer::run()?;
                        enable_raw_mode()?;
                    }

                    KeyCode::Char('q') => break,

                    _ => {}
                },

                Mode::Insert => match key.code {
                    KeyCode::Char(c) => {
                        while editor.cursor.y as usize >= editor.lines.len() {
                            editor.lines.push(String::new());
                        }

                        let line = &mut editor.lines[editor.cursor.y as usize];

                        if editor.cursor.x as usize > line.len() {
                            editor.cursor.x = line.len() as u16;
                        }

                        line.insert(editor.cursor.x as usize, c);

                        editor.cursor.x += 1;
                    }

                    KeyCode::Backspace => {
                        let y = editor.cursor.y as usize;
                        let x = editor.cursor.x as usize;

                        if y < editor.lines.len() && x > 0 {
                            let line = &mut editor.lines[y];
                            line.remove(x - 1);
                            editor.cursor.x -= 1;
                        }
                    }

                    KeyCode::Enter => {
                        editor.cursor.y += 1;
                        editor.cursor.x = 0;
                    }

                    KeyCode::Esc => editor.mode = Mode::Normal,

                    _ => {}
                },
            }
        }
    }

    disable_raw_mode()?;

    Ok(())

}
