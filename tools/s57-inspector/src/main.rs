use seatracker_provider_s57::S57Provider;
use std::{env, fs};

fn main() -> Result<(), String> {
    let path = env::args().nth(1).ok_or("usage: seatracker-s57-inspector <cell.000>")?;
    let bytes = fs::read(&path).map_err(|error| error.to_string())?;
    let inventory = S57Provider::inventory(&bytes, 100_000);
    println!("{inventory:#?}");
    Ok(())
}
