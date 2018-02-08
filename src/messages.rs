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

enum MetaMessage {
    Error(String, MessageType),
    Chat(String),
    EnterRoom(String),
}

// First byte datatypes
#[derive(Debug)]
enum MessageType {
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
        meta: first & 0x80 == 1,
        error: first & 0x40 == 1,
        info: first & 0x20 == 1,
        from_server: first & 0x10 == 1,
    }
}

// Low level parsing primitives
pub fn parse_unbounded<I>(it: &mut I) -> Option<u32>
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

pub fn parse_point<I>(it: &mut I, width: u32) -> Option<Point>
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
pub fn parse_update<I>(it: &mut I) -> Option<GameMessage>
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

pub fn parse_move<I>(it: &mut I) -> Option<GameMessage>
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
fn parse_GameMessage<I>(typ: MessageType, data: &mut I) -> Option<GameMessage>
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


pub struct Error {
    text: &'static str,
    code: MessageType,
}

pub fn handle_message(data: Vec<u8>) -> Result<(), Error> {
    let mut it = data.into_iter();
    let first = it.next().unwrap_or(0);
    let meta = meta_from_byte(first);
    if meta.from_server {
        return Err(Error {
            text: "All messages to the server must have the first bit set to 0",
            code: MessageType::MalformedMessage,
        });
    }

    let msg_type = match parse_message_type(meta.clone(), first) {
        Some(msg) => msg,
        None => {
            return Err(Error {
                text: "Malformed first byte: MessageType unknown",
                code: MessageType::MalformedMessage,
            })
        }
    };
    println!("{:?}, {:?} ", msg_type, meta);
    if !meta.meta {
        let game_message = match parse_GameMessage(msg_type, &mut it){
		Some(msg)=>msg,
		None => return Err(Error{text: "Malformed GameMessage data: Failed to parse the data according to the message type",
					 code: MessageType::MalformedMessage}),
		
		};
        println!("{:?}", game_message);
    }

    Ok(())
}
