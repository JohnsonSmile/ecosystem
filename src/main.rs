use anyhow::Context;
use ecosystem::Error;
use std::fs;

fn main() -> Result<(), anyhow::Error> {
    println!("Hello, world!");
    let filename = "non-existent-file.txt";
    let _fd = fs::File::open(filename).with_context(|| format!("Failed to open {}", filename))?; // from io::Error for Error
    fail_with_error()?;
    Ok(())
}

fn fail_with_error() -> Result<(), Error> {
    Err(Error::Custom("This is a custom error".into()))
}
