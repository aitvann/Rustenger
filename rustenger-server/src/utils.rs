use crate::room::{Error, Result};
use futures::stream::StreamExt;
use rustenger_shared::{
    codec::ServerCodec,
    message::{ClientMessage, Command},
};
use std::{
    collections::hash_map::{Entry, OccupiedEntry, VacantEntry},
    result,
};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

/// initializes the logger as follows:
///     - user messenged -> 'messages'
///     - logs level Info without user messenges -> 'general'
///     - all logs without user messenges -> stdout
pub fn init_logger<O1, O2>(messages: O1, general: O2) -> result::Result<(), log::SetLoggerError>
where
    O1: Into<fern::Output>,
    O2: Into<fern::Output>,
{
    use chrono::Local;
    use fern::{
        colors::{Color, ColoredLevelConfig},
        Dispatch,
    };
    use log::LevelFilter;
    use std::io;

    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::BrightWhite)
        .trace(Color::Magenta);

    Dispatch::new()
        .chain(
            Dispatch::new()
                .format(|out, msg, _record| {
                    out.finish(format_args!(
                        "{}: {}",
                        Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                        msg,
                    ))
                })
                .level(log::LevelFilter::Off)
                .level_for("messenges", LevelFilter::Info)
                .chain(messages),
        )
        .chain(
            Dispatch::new()
                .format(|out, msg, record| {
                    out.finish(format_args!(
                        "{}: [{} - {}] {}",
                        Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                        record.target(),
                        record.level(),
                        msg,
                    ));
                })
                .level(LevelFilter::Info)
                .filter(|md| md.target() != "messenges")
                .chain(general),
        )
        .chain(
            Dispatch::new()
                .format(move |out, msg, record| {
                    out.finish(format_args!(
                        "{}: [{} - {}] {}",
                        Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                        record.target(),
                        colors.color(record.level()),
                        msg,
                    ));
                })
                .level(LevelFilter::Trace)
                .filter(|md| md.target() != "messenges")
                .chain(io::stdout()),
        )
        .apply()?;

    Ok(())
}

/// read from framed stream
pub async fn framed_read(framed: &mut Framed<TcpStream, ServerCodec>) -> Result<ClientMessage> {
    framed
        .next()
        .await
        .unwrap_or_else(|| {
            log::error!("failed to read from framed");
            Ok(ClientMessage::Command(Command::Exit))
        })
        .map_err(Error::Bincode)
}

/// transforms the `Entry<'a, K, V>` into a `Option<Occupiedentry<'a, K, V>` or into a `Option<VacantEntry<'a, K, V>`,
pub trait EntryExt<'a, K, V> {
    fn occupied(self) -> Option<OccupiedEntry<'a, K, V>>;
    fn vacant(self) -> Option<VacantEntry<'a, K, V>>;
}

impl<'a, K, V> EntryExt<'a, K, V> for Entry<'a, K, V> {
    fn occupied(self) -> Option<OccupiedEntry<'a, K, V>> {
        match self {
            Entry::Occupied(e) => Some(e),
            Entry::Vacant(_) => None,
        }
    }

    fn vacant(self) -> Option<VacantEntry<'a, K, V>> {
        match self {
            Entry::Occupied(_) => None,
            Entry::Vacant(e) => Some(e),
        }
    }
}

pub trait ResultExt<T, E> {
    fn inspect<F>(self, f: F) -> Self
    where
        F: FnOnce(&T);
    fn inspect_err<F>(self, f: F) -> Self
    where
        F: FnOnce(&E);
}

impl<T, E> ResultExt<T, E> for result::Result<T, E> {
    fn inspect<F>(self, f: F) -> Self
    where
        F: FnOnce(&T),
    {
        if let Ok(ref o) = self {
            (f)(o);
        }

        self
    }

    fn inspect_err<F>(self, f: F) -> Self
    where
        F: FnOnce(&E),
    {
        if let Err(ref e) = self {
            (f)(e);
        }

        self
    }
}
