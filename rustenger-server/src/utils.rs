use std::collections::hash_map::{Entry, OccupiedEntry, VacantEntry};

/// initializes the logger as follows:
///     - user messenged -> 'messages'
///     - logs level Info without user messenges -> 'general'
///     - all logs without user messenges -> stdout
pub fn init_logger<O1, O2>(messages: O1, general: O2) -> Result<(), log::SetLoggerError>
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

/// converts from Entry to Option
pub trait EntryCheck {
    type OccupiedOutput;
    type VacantOutput;

    fn occupied(self) -> Option<Self::OccupiedOutput>;
    fn vacant(self) -> Option<Self::VacantOutput>;
}

impl<'a, K, V> EntryCheck for Entry<'a, K, V> {
    type OccupiedOutput = OccupiedEntry<'a, K, V>;
    type VacantOutput = VacantEntry<'a, K, V>;

    fn occupied(self) -> Option<Self::OccupiedOutput> {
        match self {
            Entry::Occupied(e) => Some(e),
            Entry::Vacant(_) => None,
        }
    }

    fn vacant(self) -> Option<Self::VacantOutput> {
        match self {
            Entry::Occupied(_) => None,
            Entry::Vacant(e) => Some(e),
        }
    }
}

impl<'a, K, V> EntryCheck for &'a Entry<'a, K, V> {
    type OccupiedOutput = &'a OccupiedEntry<'a, K, V>;
    type VacantOutput = &'a VacantEntry<'a, K, V>;

    fn occupied(self) -> Option<Self::OccupiedOutput> {
        match self {
            Entry::Occupied(e) => Some(e),
            Entry::Vacant(_) => None,
        }
    }

    fn vacant(self) -> Option<Self::VacantOutput> {
        match self {
            Entry::Occupied(_) => None,
            Entry::Vacant(e) => Some(e),
        }
    }
}

impl<'a, K, V> EntryCheck for &'a mut Entry<'a, K, V> {
    type OccupiedOutput = &'a mut OccupiedEntry<'a, K, V>;
    type VacantOutput = &'a mut VacantEntry<'a, K, V>;

    fn occupied(self) -> Option<Self::OccupiedOutput> {
        match self {
            Entry::Occupied(e) => Some(e),
            Entry::Vacant(_) => None,
        }
    }

    fn vacant(self) -> Option<Self::VacantOutput> {
        match self {
            Entry::Occupied(_) => None,
            Entry::Vacant(e) => Some(e),
        }
    }
}
