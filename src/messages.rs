// Structs for low level parsing
#[derive(Debug)]
pub struct Move(Point, u8, u32);

#[derive(Debug)]
pub struct Point(u32, u32);

// Full Messages
#[derive(Debug)]
pub enum GameMessage {
    Update(u8, u8, Vec<Point>, Vec<u32>),
    MoveMessage(u32, Move),
    Replay(u8, u32, Vec<Move>, Vec<u32>),
}

#[derive(Debug)]
pub enum MetaMessage {
    Error(String, MessageType),
    Chat(String),
    EnterRoom(String),
    Login(String),
    Info(String),
    Ping,
}

#[derive(Debug)]
pub enum Message {
    Game(GameMessage),
    Meta(MetaMessage),
}

// First byte datatypes
#[derive(Debug)]
pub enum MessageType {
    Login,
    EnterRoom,
    Ping,
    Info,
    MalformedMessage,
    FailedLogin,
    BadConnection,

    UpdateCell,
    Move,
    Status,
    Replay,
    Map,
    MissingData,
    Cheating,
    Internal,
}

#[derive(Clone, Debug)]
struct MetaData {
    info: bool,
    error: bool,
    meta: bool,
    from_server: bool,
}

// Functions to marshal in between a byte and the first byte metadata
fn parse_message_type(meta: MetaData, id: u8) -> Option<MessageType> {
    if meta.meta {
        Some(match id {
            1 => MessageType::Login,
            2 => MessageType::EnterRoom,
            3 => MessageType::Ping,
            4 => MessageType::Info,
            _ => return None,
        })
    } else {
        Some(match id {
            1 => MessageType::UpdateCell,
            2 => MessageType::Move,
            3 => MessageType::Status,
            4 => MessageType::Replay,
            _ => return None,
        })
    }
}

fn meta_from_byte(first: u8) -> MetaData {
    MetaData {
        from_server: first & 0x80 != 0,
        meta: first & 0x40 != 0,
        info: first & 0x20 != 0,
        error: first & 0x10 != 0,
    }
}

// Low level parsing primitives
fn parse_unbounded<I>(it: &mut I) -> Option<u32>
where
    I: Iterator<Item = u8>,
{
    parse_unbounded_private(it.next()?, it)
}

fn parse_unbounded_private<I>(first: u8, it: &mut I) -> Option<u32>
where
    I: Iterator<Item = u8>,
{
    Some(if first == std::u8::MAX {
        it.next()? as u32 + ((std::u8::MAX as u32) + 1) * parse_unbounded(it)?
    } else {
        first as u32
    })
}

fn parse_point<I>(it: &mut I, width: u32) -> Option<Point>
where
    I: Iterator<Item = u8>,
{
    parse_point_private(it.next()?, it, width)
}

fn parse_point_private<I>(first: u8, it: &mut I, width: u32) -> Option<Point>
where
    I: Iterator<Item = u8>,
{
    let num = parse_unbounded_private(first, it)?;
    Some(Point(num % width, num / width))
}

fn parse_move_private<I>(it: &mut I, width: u32) -> Option<Move>
where
    I: Iterator<Item = u8>,
{
    let point = parse_point(it, width)?;
    let direction = it.next()?;
    let units = parse_unbounded(it)?;
    Some(Move(point, direction, units))
}

// Parsing for full GameMessages of a particular type
fn parse_update<I>(it: &mut I) -> Option<GameMessage>
where
    I: Iterator<Item = u8>,
{
    let player = it.next()?;
    let width = it.next()?;
    let mut points = Vec::new();
    let mut values = Vec::new();
    loop {
        let first = match it.next() {
            Some(val) => val,
            None => break,
        };
        let loc = parse_point_private(first, it, width as u32)?;
        let value = parse_unbounded(it)?;
        points.push(loc);
        values.push(value);
    }
    Some(GameMessage::Update(player, width, points, values))
}

fn parse_move<I>(it: &mut I) -> Option<GameMessage>
where
    I: Iterator<Item = u8>,
{
    let width = it.next()? as u32;
    Some(GameMessage::MoveMessage(
        width,
        parse_move_private(it, width)?,
    ))
}

fn parse_replay<I>(it: &mut I) -> Option<GameMessage>
where
    I: Iterator<Item = u8>,
{
    let player = it.next()?;
    let width = it.next()? as u32;
    let mut moves = Vec::new();
    let mut skips = Vec::new();

    loop {
        let first = match it.next() {
            Some(val) => val,
            None => break,
        };
        let skip = parse_unbounded_private(first, it)?;
        let mov = parse_move_private(it, width)?;
        skips.push(skip);
        moves.push(mov);
    }

    Some(GameMessage::Replay(player, width, moves, skips))
}

// Highest level GameMessage parser
fn parse_game_message<I>(typ: MessageType, data: &mut I) -> Option<GameMessage>
where
    I: Iterator<Item = u8>,
{
    match typ {
        MessageType::UpdateCell => parse_update(data),
        MessageType::Move => parse_move(data),
        MessageType::Replay => parse_replay(data),
        _ => None,
    }
}

fn parse_string<I>(data: &mut I) -> Option<String>
where
    I: Iterator<Item = u8>,
{
    let len = parse_unbounded(data)? as usize;
    let strdata: Vec<u8> = data.take(len).collect();
    if len == strdata.len() {
        Some(std::str::from_utf8(&strdata).ok()?.to_string())
    } else {
        None
    }
}

fn parse_meta_message<I>(typ: MessageType, data: &mut I) -> Option<MetaMessage>
where
    I: Iterator<Item = u8>,
{
    Some(match typ {
        MessageType::Login => MetaMessage::Login(parse_string(data)?),
        MessageType::EnterRoom => MetaMessage::EnterRoom(parse_string(data)?),
        MessageType::Ping => MetaMessage::Ping,
        MessageType::Info => MetaMessage::Info(parse_string(data)?),
        _ => return None,
    })
}

#[derive(Debug)]
pub struct Error {
    text: &'static str,
    code: MessageType,
}

pub fn parse_message(data: Vec<u8>) -> Result<Message, Error> {
    let mut it = data.into_iter();
    let first = it.next().unwrap_or(0);
    let meta = meta_from_byte(first);
    if meta.from_server {
        return Err(Error {
            text: "All messages to the server must have the first bit set to 0",
            code: MessageType::MalformedMessage,
        });
    }
    let msg_type = match parse_message_type(meta.clone(), first & 0xF) {
        Some(msg) => msg,
        None => {
            return Err(Error {
                text: "Malformed first byte: MessageType unknown",
                code: MessageType::MalformedMessage,
            })
        }
    };
    Ok(if meta.meta {
        Message::Meta(match parse_meta_message(msg_type,&mut it){
		Some(msg)=>msg,
		None => return Err(Error{text: "Malformed MetaMessage data: Failed to parse the data according to the message type",
					 code: MessageType::MalformedMessage}),
		})
    } else {
        Message::Game(match parse_game_message(msg_type, &mut it){
		Some(msg)=>msg,
		None => return Err(Error{text: "Malformed GameMessage data: Failed to parse the data according to the message type",
					 code: MessageType::MalformedMessage}),
		})
    })
}
