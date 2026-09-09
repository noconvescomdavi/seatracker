use std::{env, fs, thread, time::Duration};

fn main() -> Result<(), String> {
    let path = env::args()
        .nth(1)
        .ok_or("usage: seatracker-nmea-replay <log> [speed]")?;
    let speed: f64 = env::args()
        .nth(2)
        .and_then(|value| value.parse().ok())
        .unwrap_or(1.0);
    if !matches!(speed, 0.5 | 1.0 | 2.0 | 5.0 | 10.0) {
        return Err("speed must be 0.5, 1, 2, 5 or 10".into());
    }
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let delay = Duration::from_secs_f64(1.0 / speed);
    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        match seatracker_nmea::parse(line) {
            Ok(message) => println!("{message:?}"),
            Err(error) => eprintln!("INVALID: {error}: {line}"),
        }
        thread::sleep(delay);
    }
    Ok(())
}
