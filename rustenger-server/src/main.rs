#![feature(async_closure)]
use tokio::{net::TcpListener, prelude::*};

mod client;
mod room;
mod utils;

const DEFAULT_ADDR: &str = "0.0.0.0:4732";
const PATH_TO_MESENGES_LOG: &str = "messenges.log";
const PATH_TO_GENERAL_LOG: &str = "general.log";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    use futures::stream::{self, StreamExt};

    let messenges = fern::log_file(PATH_TO_MESENGES_LOG)?;
    let general = fern::log_file(PATH_TO_GENERAL_LOG)?;
    utils::init_logger(messenges, general).expect("failed to initialize logger");

    // accepts several possible addresses
    let matches = clap::App::new("Rustenger server")
        .version("0.0.0")
        .author("Aitzhanov Ivan <aitvann@gmail.com>")
        .about("Asynchronous server for Rustenger")
        .arg(
            clap::Arg::with_name("addresses")
                .short("a")
                .multiple(true)
                .takes_value(true)
                .help("address of server")
                .default_value(DEFAULT_ADDR),
        )
        .get_matches();

    // selects the first available address from the arguments
    let addrs = matches.values_of("addresses").unwrap();
    let stream = stream::iter(addrs).filter_map(async move |a| {
        TcpListener::bind(a)
            .await
            .map_err(|e| log::warn!("failed to bind to address: {}; error: {}", a, e))
            .ok()
    });
    futures::pin_mut!(stream);
    let listener = stream
        .next()
        .await
        .expect("failed to select listener address");

    log::info!(
        "listener has successful bind to address: {}",
        listener.local_addr().unwrap()
    );

    Ok(())
}
