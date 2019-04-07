use failure::Error;
use std::io::BufRead;
use std::io::BufReader;
use std::net::TcpStream;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "dump-net",
    about = "Parse messages from a TCP server in AVR format."
)]
struct Cli {
    #[structopt(help = "host")]
    host: String,
    #[structopt(help = "port", default_value = "30002")]
    port: u16,
}

fn main() -> Result<(), Error> {
    let args = Cli::from_args();
    let addr = format!("{}:{}", &args.host, &args.port);
    let stream = TcpStream::connect(&addr)?;
    let reader = BufReader::new(stream);
    println!("Connected to {}", &addr);
    for line in reader.lines() {
        let frame = line?;
        match adsb::parse_avr(&frame) {
            Ok((message, _)) => println!("{} {:#?}", frame, message),
            Err(error) => println!("{} {:#?}", frame, error),
        }
    }
    Ok(())
}
