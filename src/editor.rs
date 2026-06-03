use crate::explorer;

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode},
    execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, size},
};
use std::io::{self, Write, stdout};
use std::path::{Path, PathBuf};

pub struct Cursor {
    x: u16,
    y: u16,
}

pub enum Mode {
    Normal,
    Insert,
    Open,
}

pub struct Editor {
    cursor: Cursor,
    mode: Mode,
    lines: Vec<String>,
    file_path: Option<PathBuf>,
}

impl Editor {
    fn clamp_cursor(&mut self) {
        if self.cursor.y as usize >= self.lines.len() {
            self.cursor.y = (self.lines.len().saturating_sub(1)) as u16;
        }

        let line_len = self
            .lines
            .get(self.cursor.y as usize)
            .map(|l| l.len())
            .unwrap_or(0);

        if self.cursor.x as usize > line_len {
            self.cursor.x = line_len as u16;
        }
    }
}

pub fn run() -> io::Result<()> {
    let mut editor = Editor {
        cursor: Cursor { x: 0, y: 0 },
        mode: Mode::Normal,
        lines: vec![String::new()],
        file_path: None,
    };

    execute!(stdout(), Clear(ClearType::All))?;

    loop {
        if !is_raw_mode_enabled()? {
            enable_raw_mode()?;
        }

        let (_, rows) = size()?;

        editor.clamp_cursor();

        let mode_text = match editor.mode {
            Mode::Normal => "-- NORMAL --",
            Mode::Insert => "-- INSERT --",
            Mode::Open => "-- OPEN --",
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
                KeyCode::Delete | KeyCode::Left => {
                    if editor.cursor.x > 0 {
                        editor.cursor.x -= 1;
                    }
                }

                KeyCode::End | KeyCode::Down => editor.cursor.y += 1,

                KeyCode::Home | KeyCode::Up => {
                    if editor.cursor.y > 0 {
                        editor.cursor.y -= 1;
                    }
                }

                KeyCode::PageDown | KeyCode::Right => editor.cursor.x += 1,

                KeyCode::PageUp => editor.mode = Mode::Normal,

                _ => {}
            }
            editor.clamp_cursor();

            match editor.mode {
                Mode::Normal => match key.code {
                    KeyCode::Insert => editor.mode = Mode::Insert,

                    KeyCode::Char(' ') => editor.mode = Mode::Open,

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

                        if y >= editor.lines.len() {
                            return Ok(());
                        }

                        if x > 0 {
                            editor.lines[y].remove(x - 1);
                            editor.cursor.x -= 1;
                        } else if y > 0 {
                            let current_line = editor.lines.remove(y);
                            editor.cursor.y -= 1;

                            let prev_line = &mut editor.lines[editor.cursor.y as usize];

                            let prev_len = prev_line.len();
                            prev_line.push_str(&current_line);

                            editor.cursor.x = prev_len as u16;
                        }
                    }

                    KeyCode::Enter => {
                        let y = editor.cursor.y as usize;
                        let x = editor.cursor.x as usize;

                        while y >= editor.lines.len() {
                            editor.lines.push(String::new());
                        }

                        let current_line = &mut editor.lines[y];

                        let new_line = current_line.split_off(x);

                        editor.lines.insert(y + 1, new_line);

                        editor.cursor.y += 1;
                        editor.cursor.x = 0;
                    }

                    _ => {}
                },
                Mode::Open => match key.code {
                    KeyCode::Char('e') => {
                        if let Some(path) = explorer::run()? {
                            editor.lines = open_file(&path)?;
                            editor.file_path = Some(path);
                            editor.cursor.x = 0;
                            editor.cursor.y = 0;
                        }
                        editor.mode = Mode::Normal;
                    }

                    _ => {}
                },
            }
        }
    }

    disable_raw_mode()?;

    Ok(())
}
fn open_file(path: &Path) -> io::Result<Vec<String>> {
    let content = std::fs::read_to_string(path)?;
    Ok(content.lines().map(|s| s.to_string()).collect())
}
