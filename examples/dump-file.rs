use clap::Parser;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Parser, Debug)]
#[command(about = "Parse messages from a file in AVR format.")]
struct Cli {
    #[arg(help = "Path")]
    path: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
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
