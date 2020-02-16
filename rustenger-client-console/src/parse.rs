use arrayvec::ArrayString;
use rustenger_shared::{
    account::Color,
    message::{ClientMessage, Command, UserMessage},
    RoomName,
};
use std::str::FromStr;
use thiserror::Error;

/// parse input with following format:
///     * [TEXT] = UserMessage
///     * :[COMMAND SHORT NAME] [ARG..] = Command -- one character, may be not all commands are avaliabel
///     * ::[COMMAND FULL NAME] [ARG..] = Command -- muiltiple character, all commands are avaliable
pub fn parse_input(buffer: &str) -> Result<ClientMessage, Error> {
    let client_message = if buffer.starts_with(":") {
        let cmd = parse_command(&buffer[1..])?;
        ClientMessage::Command(cmd)
    } else {
        let msg = parse_user_message(buffer)?;
        ClientMessage::UserMessage(msg)
    };

    Ok(client_message)
}

/// parse user message
fn parse_user_message(buffer: &str) -> Result<UserMessage, Error> {
    let text = ArrayString::from(buffer).unwrap();
    Ok(text)
}

macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}

macro_rules! count_tts {
    ($($tts:tt)*) => {<[()]>::len(&[$(replace_expr!($tts ())),*])};
}

/// example: parse_args!(iter => LogIn: ArrayString, ArrayString);
macro_rules! parse_args {
    ($input:expr => $cmd:ident: $( $arg:ty ),+ ) => {{
        let expected = count_tts!( $( $arg )+ );
        check_args_num($input, expected)?;

        let mut iter = $input.split_whitespace();
        Command::$cmd(
            $( <$arg as FromStr>::from_str(iter.next().unwrap())
                .map_err(|e| Error::Parse(Box::new(e)))? ),+
        )
    }};

    ($input:expr => $cmd:ident) => {{
        check_args_num($input, 0)?;
        Command::$cmd
    }};
}

/// checks if the number of arguments matches the expected
fn check_args_num(input: &str, expected: usize) -> Result<(), Error> {
    let found = input.split_whitespace().count();
    if expected != found {
        return Err(Error::InvalidArgumentNum { expected, found });
    }

    Ok(())
}

/// parse command
fn parse_command(buffer: &str) -> Result<Command, Error> {
    let (cmd_name, args) = if let Some(pos) = buffer.find(' ') {
        let (cmd_name, args) = buffer.split_at(pos);
        (cmd_name, &args[1..])
    } else {
        (buffer, "")
    };

    let cmd = match cmd_name {
        "c" | ":CreateRoom" => parse_args!(args => CreateRoom: RoomName),
        "s" | ":SelectRoom" => parse_args!(args => SelectRoom: RoomName),
        "e" | ":ExitRoom" => parse_args!(args => ExitRoom),
        "l" | ":RoomsList" => parse_args!(args => RoomsList),
        ":SelectColor" => parse_args!(args => SelectColor: Color),
        "d" | ":DeleteAccount" => parse_args!(args => DeleteAccount),
        "q" | ":Quit" => parse_args!(args => Exit), // TODO: rename in the server
        _ => return Err(Error::InvalidCommandName),
    };

    Ok(cmd)
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("parse error: {0}")]
    Parse(#[from] Box<dyn std::error::Error>),
    #[error("this command takes {expected} parameter but {found} parameters were supplied")]
    InvalidArgumentNum { expected: usize, found: usize },
    #[error("invalid command name")]
    InvalidCommandName,
}
