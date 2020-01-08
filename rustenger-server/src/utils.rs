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
        Dispatch, FormatCallback,
    };
    use log::{LevelFilter, Record};
    use std::{fmt::Arguments, io};

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
