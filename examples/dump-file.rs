use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
// use std::time::Duration;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "dump-file",
    about = "Parse messages from a file in AVR format."
)]
struct Cli {
    #[structopt(help = "path")]
    path: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();
    let file = File::open(&args.path)?;
    let lines = BufReader::new(file).lines();
    for line in lines {
        let frame = line?;
        match adsb::parse_avr(&frame) {
            Ok((message, _)) => println!("{} {:#?}", frame, message),
            Err(error) => println!("{} {:#?}", frame, error),
        }
    }

    Ok(())
}
