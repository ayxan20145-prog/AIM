use colored::Colorize;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    execute,
    style::Print,
    terminal::{Clear, ClearType, disable_raw_mode},
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::{env, process::Command};

pub fn run() -> io::Result<Option<std::path::PathBuf>> {
    let mut stdout = io::stdout();
    execute!(stdout, Hide, Clear(ClearType::All))?;

    let mut selected: usize = 0;

    let mut dir = env::current_dir()?;

    let mut show_hidden = false;

    loop {
        let mut entries_list: Vec<(PathBuf, bool)> = Vec::new();

        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(e) => {
                execute!(stdout, Print(format!("Error: {e}\r\n")))?;
                thread::sleep(Duration::from_secs(1));
                dir.pop();
                continue;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let is_dir = path.is_dir();
            let name = path.file_name().unwrap_or_default().to_string_lossy();

            if !show_hidden {
                if name.starts_with(".") {
                    continue;
                }
            }

            entries_list.push((path, is_dir));
        }

        if selected >= entries_list.len() && !entries_list.is_empty() {
            selected = entries_list.len() - 1;
        }

        execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;

        execute!(stdout, Print(format!("{}\r\n", dir.display())))?;

        for (i, (path, is_dir)) in entries_list.iter().enumerate() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();

            if i == selected {
                if *is_dir {
                    execute!(stdout, Print(format!("> {}/\r\n", name.blue())))?;
                } else {
                    execute!(stdout, Print(format!("> {}\r\n", name)))?;
                }
            } else {
                if *is_dir {
                    execute!(stdout, Print(format!("  {}/\r\n", name.blue())))?;
                } else {
                    execute!(stdout, Print(format!("  {}\r\n", name)))?;
                }
            }
        }

        stdout.flush()?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::End | KeyCode::Down => {
                        if selected + 1 < entries_list.len() {
                            selected += 1;
                        }
                    }
                    KeyCode::Home | KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Delete | KeyCode::Left => {
                        dir.pop();
                    }
                    KeyCode::PageDown | KeyCode::Right => {
                        if let Some((path, is_dir)) = entries_list.get(selected) {
                            if *is_dir {
                                dir = path.clone();
                                selected = 0;
                            }
                        }
                    }
                    KeyCode::Char('a') => {
                        create_dir(&dir)?;
                    }
                    KeyCode::Char('f') => {
                        create_file(&dir)?;
                    }
                    KeyCode::Char('d') => {
                        if let Some((path, is_dir)) = entries_list.get(selected) {
                            delete(path, *is_dir)?;
                        }
                    }
                    KeyCode::Char('c') => {
                        if let Some((path, is_dir)) = entries_list.get(selected) {
                            if *is_dir {
                                copy_dir(path)?;
                            } else {
                                copy_file(path)?;
                            }
                        }
                    }
                    KeyCode::Char('m') => {
                        if let Some((path, _is_dir)) = entries_list.get(selected) {
                            movee(path)?;
                        }
                    }
                    KeyCode::Char('r') => {
                        if let Some((path, _is_dir)) = entries_list.get(selected) {
                            rename(path)?;
                        }
                    }
                    KeyCode::Char('.') => {
                        show_hidden = !show_hidden;
                        selected = 0;
                    }
                    KeyCode::Enter => {
                        if let Some((path, is_dir)) = entries_list.get(selected) {
                            if !*is_dir {
                                cleanup(&mut stdout)?;

                                return Ok(Some(path.clone()));
                            }
                        }
                    }
                    KeyCode::Char('t') => {
                        open_terminal(&dir);
                    }
                    KeyCode::Char('?') => {
                        execute!(
                            stdout,
                            Print(format!(
                                "a -> Create dir\r\n\
                                 f -> Create file\r\n\
                                 d -> Delte\r\n\
                                 c -> Copy\r\n\
                                 m -> Move\r\n\
                                 r -> Rename\r\n\
                                 . -> Toggle hidden\r\n\
                                 enter -> Open in editor\r\n\
                                 t -> Open terminal here\r\n\
                                 q -> Quit"
                            ))
                        )?;
                        pause();
                    }
                    KeyCode::Char('q') => {
                        cleanup(&mut stdout)?;
                        disable_raw_mode()?;
                        return Ok(None);
                    }
                    _ => {}
                }
            }
        }
    }
}
pub fn cleanup(stdout: &mut std::io::Stdout) -> io::Result<()> {
    execute!(stdout, Show)?;
    Ok(())
}
pub fn create_dir(path: &Path) -> io::Result<()> {
    let mut name = String::new();

    print!("Enter the directory name: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut name)?;
    let name = name.trim();

    if name.is_empty() {
        return Ok(());
    }

    let full_path = path.join(name);

    fs::create_dir(full_path)?;

    Ok(())
}
pub fn create_file(path: &Path) -> io::Result<()> {
    let mut name = String::new();

    print!("Enter the file name: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut name)?;
    let name = name.trim();

    if name.is_empty() {
        return Ok(());
    }

    let full_path = path.join(name);

    if full_path.exists() {
        if full_path.is_dir() {
            println!("Error: a directory with this name already exists");
        } else {
            println!("Error: file already exists");
        }
        pause();
        return Ok(());
    }

    fs::write(full_path, "")?;

    Ok(())
}
pub fn delete(path: &Path, is_dir: bool) -> io::Result<()> {
    print!(
        "Are you sure want to delete: {}? (y/n)",
        path.file_name().unwrap_or_default().to_string_lossy()
    );
    io::stdout().flush()?;

    if let Event::Key(key) = event::read()? {
        match key.code {
            KeyCode::Char('y') => {
                let result = if is_dir {
                    fs::remove_dir_all(path)
                } else {
                    fs::remove_file(path)
                };

                result?;
            }
            _ => return Ok(()),
        }
    }
    Ok(())
}
pub fn copy_file(path: &Path) -> io::Result<()> {
    let mut dest = String::new();

    print!("Enter the destination: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut dest)?;

    let dest = dest.trim();
    let dest_path = Path::new(dest);

    fs::copy(path, dest_path)?;

    Ok(())
}
pub fn copy_dir(path: &Path) -> io::Result<()> {
    let mut dest = String::new();

    print!("Enter the destination: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut dest)?;

    let dest = dest.trim();
    let dest_path = Path::new(dest);

    copy_dir_recursive(path, dest_path)?;

    Ok(())
}
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
pub fn movee(path: &Path) -> io::Result<()> {
    let mut dest = String::new();

    print!("Enter the destination: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut dest)?;

    let dest = dest.trim();
    let dest_path = Path::new(dest);

    fs::rename(path, dest_path)?;

    Ok(())
}
pub fn rename(path: &Path) -> io::Result<()> {
    let mut name = String::new();

    print!("Enter the name: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut name)?;

    let name = name.trim();

    let parent = path.parent().unwrap_or(Path::new("."));
    let new_path = parent.join(name);

    fs::rename(path, new_path)?;

    Ok(())
}
pub fn open_terminal(dir: &Path) {
    if Command::new("xdg-terminal-exec")
        .arg("--dir")
        .arg(dir)
        .spawn()
        .is_ok()
    {
        return;
    }
    if Command::new("ptyxis")
        .arg("--working-directory")
        .arg(dir)
        .arg("--new-window")
        .spawn()
        .is_ok()
    {
        return;
    }

    let cmds: &[(&str, &[&str])] = &[
        ("x-terminal-emulator", &[]),
        ("gnome-terminal", &["--working-directory"]),
        ("konsole", &["--workdir"]),
        ("xfce4-terminal", &["--working-directory"]),
        ("kitty", &["--directory"]),
        ("alacritty", &["--working-directory"]),
        ("wezterm", &["start", "--cwd"]),
        ("xterm", &[]),
    ];

    for (term, args) in cmds {
        let mut command = Command::new(term);

        if args.is_empty() {
            command.current_dir(dir);
        } else {
            command.args(*args).arg(dir);
        }

        if command.spawn().is_ok() {
            return;
        }
    }

    println!("Failed to open terminal");
    pause();
}
pub fn pause() {
    let mut pause = String::new();
    io::stdin().read_line(&mut pause).unwrap();
}
