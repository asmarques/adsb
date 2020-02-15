use std::io::BufRead;
use std::io::BufReader;
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;
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
    #[structopt(long = "timeout", help = "connection timeout", default_value = "30")]
    timeout: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();
    let addr = format!("{}:{}", &args.host, &args.port).parse::<SocketAddr>()?;
    let timeout = Duration::from_secs(args.timeout);
    let stream = TcpStream::connect_timeout(&addr, timeout)?;
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
