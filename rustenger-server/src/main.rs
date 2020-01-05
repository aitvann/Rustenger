#![feature(async_closure)]
use tokio::{net::TcpListener, prelude::*};

mod utils;

const DEFAULT_ADDR: &'static str = "0.0.0.0:4732";
const PATH_TO_MESENGES_LOG: &'static str = "messenges.log";
const PATH_TO_GENERAL_LOG: &'static str = "general.log";

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
    let _listener = stream::iter(addrs)
        .filter_map(async move |a| {
            TcpListener::bind(a)
                .await
                .map_err(|e| log::warn!("failed to bind to address: {}; error: {}", a, e))
                .map(|l| {
                    log::info!(
                        "listener has successful bind to address: {}",
                        l.local_addr().unwrap()
                    )
                })
                .ok()
        })
        .boxed()
        .next()
        .await
        .expect("failed to select listener address");

    Ok(())
}
