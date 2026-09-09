use seatracker_chart_engine::{inspect_chart, positively_identified_format};
use std::{env, fs};

fn main() -> Result<(), String> {
    let path = env::args()
        .nth(1)
        .ok_or("usage: seatracker-chart-inspector <chart>")?;
    let bytes = fs::read(&path).map_err(|error| error.to_string())?;
    let inspection = inspect_chart(&path, &bytes);
    println!("inspection={inspection:#?}");
    println!(
        "positive_format={:?}",
        positively_identified_format(&inspection)
    );
    Ok(())
}
