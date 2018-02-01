#[derive(Debug)]
pub enum GameMessage {
    Update(u8, u8, Vec<Point>, Vec<u32>),
    MoveMessage(u32, Move),
    Replay(u8, u32, Vec<Move>, Vec<u32>),
}

enum MetaMessage {
    other,
}

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

fn parse_message_type(meta: MetaData, id: u8) -> Option<MessageType> {
    if meta.meta {
        Some(match id {
            1 => Login,
            2 => EnterRoom,
            3 => Ping,
            4 => Info,
            _ => return None,
        })
    } else {
        Some(match id {
            1 => Updatecell,
            2 => Move,
            3 => Status,
            4 => Replay,
            _ => return None,
        })
    }
}

struct MetaData {
    info: bool,
    error: bool,
    meta: bool,
    from_server: bool,
}

pub struct Error {
    text: &'static str,
    code: MessageType,
}
fn parse_GameMessage(typ: MessageType, data: Vec<u8>) -> Option<GameMessage> {
    match typ {
        MessageType::UpdateCell => parse_update(data),
        MessageType::Move => parse_move(data),
        MessageType::Replay => parse_replay(data),
        _ => None,
    }
}

pub fn parse_update(data: Vec<u8>) -> Option<GameMessage> {
    let mut it = data.into_iter();
    let player = it.next()?;
    let width = it.next()?;
    let mut points = Vec::new();
    let mut values = Vec::new();
    loop {
        let first = match it.next() {
            Some(val) => val,
            None => break,
        };
        let loc = parse_point_private(first, &mut it, width as u32)?;
        let value = parse_unbounded(&mut it)?;
        points.push(loc);
        values.push(value);
    }
    Some(GameMessage::Update(player, width, points, values))
}

pub fn parse_move(data: Vec<u8>) -> Option<GameMessage> {
    let mut it = data.into_iter();
    let width = it.next()? as u32;
    Some(GameMessage::MoveMessage(
        width,
        parse_move_private(&mut it, width)?,
    ))
}

#[derive(Debug)]
pub struct Move(Point, u8, u32);

fn parse_move_private<I>(it: &mut I, width: u32) -> Option<Move>
where
    I: Iterator<Item = u8>,
{
    let point = parse_point(it, width)?;
    let direction = it.next()?;
    let units = parse_unbounded(it)?;
    Some(Move(point, direction, units))
}

fn parse_replay(data: Vec<u8>) -> Option<GameMessage> {
    let mut it = data.into_iter();
    let player = it.next()?;
    let width = it.next()? as u32;
    let mut moves = Vec::new();
    let mut skips = Vec::new();

    loop {
        let first = match it.next() {
            Some(val) => val,
            None => break,
        };
        let skip = parse_unbounded_private(first, &mut it)?;
        let mov = parse_move_private(&mut it, width)?;
        skips.push(skip);
        moves.push(mov);
    }

    Some(GameMessage::Replay(player, width, moves, skips))
}
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

#[derive(Debug)]
pub struct Point(u32, u32);

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

pub fn handle_message(data: Vec<u8>) -> Result<(), Error> {
    let first = 0x4F;
    let meta = meta_from_byte(first);
    if meta.from_server {
        return Err(Error {
            text: "All messages to the server must have the first bit set to 0",
            code: MessageType::MalformedMessage,
        });
    }

    if meta.meta {
        return Err(Error {
            text: "Expected a GameMessage, but has the Meta field set",
            code: MessageType::MalformedMessage,
        });
    }

    Ok(())
}

fn meta_from_byte(first: u8) -> MetaData {
    MetaData {
        meta: first & 0x80 == 1,
        error: first & 0x40 == 1,
        info: first & 0x20 == 1,
        from_server: first & 0x10 == 1,
    }
}
