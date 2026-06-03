use std::io;

mod editor;
mod explorer;

fn main() -> io::Result<()> {
    editor::run()?;

    Ok(())
}
