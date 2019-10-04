use holo_config::{Config};

use failure::Error;
use std::io;

fn main() -> Result<(), Error> {
    let config: Config = serde_json::from_reader(io::stdin())?;

    println!("{:?}", config);

    Ok(())
}
