use colored::Colorize;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    execute,
    style::Print,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::{env, process::Command};

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;

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
                    KeyCode::Char('j') | KeyCode::Down => {
                        if selected + 1 < entries_list.len() {
                            selected += 1;
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Char('h') | KeyCode::Left => {
                        dir.pop();
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        if let Some((path, is_dir)) = entries_list.get(selected) {
                            if *is_dir {
                                dir = path.clone();
                                selected = 0;
                            }
                        }
                    }
                    KeyCode::Char('a') => {
                        disable_raw_mode()?;
                        create_dir(&dir)?;
                        enable_raw_mode()?;
                    }
                    KeyCode::Char('f') => {
                        disable_raw_mode()?;
                        create_file(&dir)?;
                        enable_raw_mode()?;
                    }
                    KeyCode::Char('d') => {
                        if let Some((path, is_dir)) = entries_list.get(selected) {
                            delete(path, *is_dir)?;
                        }
                    }
                    KeyCode::Char('c') => {
                        if let Some((path, is_dir)) = entries_list.get(selected) {
                            disable_raw_mode()?;
                            if *is_dir {
                                copy_dir(path)?;
                            } else {
                                copy_file(path)?;
                            }
                            enable_raw_mode()?;
                        }
                    }
                    KeyCode::Char('m') => {
                        if let Some((path, _is_dir)) = entries_list.get(selected) {
                            disable_raw_mode()?;
                            movee(path)?;
                            enable_raw_mode()?;
                        }
                    }
                    KeyCode::Char('r') => {
                        if let Some((path, _is_dir)) = entries_list.get(selected) {
                            disable_raw_mode()?;
                            rename(path)?;
                            enable_raw_mode()?;
                        }
                    }
                    KeyCode::Char('.') => {
                        show_hidden = !show_hidden;
                        selected = 0;
                    }
                    KeyCode::Char('o') => {
                        if let Some((path, is_dir)) = entries_list.get(selected) {
                            if !*is_dir {
                                let file_path = path.clone();
                                let editor =
                                    env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

                                disable_raw_mode()?;
                                execute!(stdout, Show)?;

                                Command::new(editor).arg(&file_path).status()?;

                                execute!(stdout, Hide, Clear(ClearType::All))?;
                                enable_raw_mode()?;
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
                                 o -> Open in editor\r\n\
                                 t -> Open terminal here\r\n\
                                 q -> Quit"
                            ))
                        )?;
                        disable_raw_mode()?;
                        pause();
                        enable_raw_mode()?;
                    }
                    KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }
    }

    execute!(stdout, Show)?;
    disable_raw_mode()?;
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
