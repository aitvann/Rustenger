use rustenger_shared::codec::ClientCodec;
use std::{
    iter,
    net::{Ipv4Addr, SocketAddr, TcpStream},
};

mod framed;
use framed::Framed;

const DEFAULT_ADDR: Ipv4Addr = Ipv4Addr::LOCALHOST;
const DEFAULT_PORT: u16 = 4732;

fn main() {
    let matches = clap::App::new("Rustenger console client")
        .version("0.0.0")
        .author("Aitzhanov Ivan <aitvann@gmail.com>")
        .about("Asynchronous server for Rustenger")
        .arg(
            clap::Arg::with_name("addresses")
                .short("a")
                .multiple(true)
                .takes_value(true)
                .help("address of server"),
        )
        .get_matches();

    let mut addrs = matches.values_of("addresses");
    let stream = addrs
        .iter_mut()
        .flatten()
        .filter_map(|a| a.parse().ok())
        .chain(iter::once((DEFAULT_ADDR, DEFAULT_PORT).into()))
        .filter_map(|a: SocketAddr| TcpStream::connect(a).ok())
        .next()
        .expect("failed to connect to server");

    let codec = ClientCodec::new();
    let mut _framed = Framed::new(stream, codec);
}
