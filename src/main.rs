use std::io;

mod editor;
mod explorer;
mod syntax;

fn main() -> io::Result<()> {
    editor::run()?;

    Ok(())
}
