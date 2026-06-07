use crate::explorer;

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, size},
};
use std::io::{self, Write, stdout};
use std::path::{Path, PathBuf};

struct Cursor {
    x: u16,
    y: u16,
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    Insert,
    Command,
}

struct Editor {
    cursor: Cursor,
    mode: Mode,
    lines: Vec<String>,
    file_path: Option<PathBuf>,
    command: String,
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
        command: String::new(),
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
            Mode::Command => "",
        };

        execute!(stdout(), Clear(ClearType::All))?;

        for (i, line) in editor.lines.iter().enumerate() {
            execute!(stdout(), MoveTo(0, i as u16))?;
            print!("{}", line);
        }

        execute!(stdout(), MoveTo(0, rows - 1), Clear(ClearType::CurrentLine),)?;

        if editor.mode == Mode::Command {
            print!(":{} ", editor.command);
        } else {
            print!("{}", mode_text);
        }

        execute!(stdout(), MoveTo(editor.cursor.x, editor.cursor.y))?;

        stdout().flush()?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                disable_raw_mode()?;
                break;
            }
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

                KeyCode::Esc => editor.mode = Mode::Normal,

                _ => {}
            }
            editor.clamp_cursor();

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

                    KeyCode::Char(':') => {
                        editor.mode = Mode::Command;
                        editor.command.clear();
                    }

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

                    KeyCode::Tab => {
                        let y = editor.cursor.y as usize;

                        while y >= editor.lines.len() {
                            editor.lines.push(String::new());
                        }

                        let line = &mut editor.lines[y];

                        let tab = "    ";

                        line.insert_str(editor.cursor.x as usize, tab);

                        editor.cursor.x += tab.len() as u16;
                    }

                    _ => {}
                },
                Mode::Command => match key.code {
                    KeyCode::Char(c) => {
                        editor.command.push(c);
                    }

                    KeyCode::Backspace => {
                        editor.command.pop();
                    }

                    KeyCode::Enter => {
                        let cmd = editor.command.trim().to_string();

                        if let Err(e) = handle_command(&cmd, &mut editor) {
                            eprintln!("Command error: {e}");
                        }

                        editor.command.clear();
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
fn handle_command(cmd: &str, editor: &mut Editor) -> io::Result<()> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();

    match parts.as_slice() {
        ["q"] => {
            std::process::exit(0);
        }

        ["w"] => {
            if let Some(path) = &editor.file_path {
                let content = editor.lines.join("\n");
                std::fs::write(path, content)?;
            }
        }

        ["wq"] => {
            if let Some(path) = &editor.file_path {
                let content = editor.lines.join("\n");
                std::fs::write(path, content)?;
            }

            std::process::exit(0);
        }

        ["Ex"] => {
            if let Some(path) = explorer::run()? {
                editor.lines = open_file(&path)?;
                editor.file_path = Some(path);
                editor.cursor.x = 0;
                editor.cursor.y = 0;
            }
        }

        _ => {}
    }

    Ok(())
}
