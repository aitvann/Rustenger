#![feature(async_closure)]
use crate::utils::ResultExt;
use futures::stream::{self, StreamExt};
use tokio::net::{TcpListener, TcpStream};

mod client;
use client::Client;

mod room;
use room::Server;

mod utils;

const DEFAULT_ADDR: &str = "0.0.0.0:4732";
const PATH_TO_MESENGES_LOG: &str = "messenges.log";
const PATH_TO_GENERAL_LOG: &str = "general.log";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // init logger
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
    let mut listener = stream
        .next()
        .await
        .expect("failed to select listener address");

    log::info!(
        "listener has successful bind to address: {}",
        listener.local_addr().unwrap()
    );

    let server = Server::new();

    let mut incoming = listener.incoming();
    while let Some(res) = incoming.next().await {
        if let Ok(stream) = res.inspect_err(|e| log::error!("failed to accept stream: {}", e)) {
            tokio::spawn(process(stream, server.clone()));
        }
    }

    Ok(())
}

/// process the accepted stream
async fn process(stream: TcpStream, server: Server) {
    if let Ok(addr) = stream
        .peer_addr()
        .inspect_err(|e| log::warn!("failed to get peer addr: {}", e))
    {
        log::info!("accept stream: {}", addr);
    }

    if let Ok(client) = Client::new(stream, server)
        .await
        .inspect_err(|e| log::error!("failed to create 'Client': {}", e))
    {
        log::info!("succefull create 'Client'");

        // if the user has not exit
        if let Some(client) = client {
            let _ = client
                .run()
                .await
                .inspect_err(|e| log::error!("error while run 'Client': {}", e));
        }
    }
}
