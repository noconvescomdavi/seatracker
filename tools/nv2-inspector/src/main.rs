use seatracker_provider_nv2::Nv2Provider;
use sha2::{Digest, Sha256};
use std::{env, fs};

fn main() -> Result<(), String> {
    let path = env::args().nth(1).ok_or("usage: seatracker-nv2-inspector <file.nv2>")?;
    let bytes = fs::read(&path).map_err(|error| error.to_string())?;
    let hash = Sha256::digest(&bytes);
    println!("bytes={}", bytes.len());
    println!("sha256={hash:x}");
    for observation in Nv2Provider::printable_strings(&bytes, 6, 100) {
        println!("{observation:?}");
    }
    Ok(())
}
