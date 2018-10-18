// A WebSocket echo server

extern crate ws;

use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use std::rc::Rc;

use ws::{listen, CloseCode, Handler, Handshake, Message, Result, Sender};
use ws::util::Token;

struct Client {
    id: Token,
    server: Rc<RefCell<Server>>,
}

impl Client {
    fn send_message(&mut self, msg: String) -> Option<()> {
        let server = &mut *self.server.borrow_mut();
        if server.inRoom.contains_key(&self.id) {
            let room_id = server.inRoom.get(&self.id)?;
            let temp = &mut "anonymous".to_string();
            let mut output = server.alias.get_mut(&self.id).unwrap_or(temp);
            output.push_str(":");
            output.push_str(&msg);
            let to_send = Message::Text(output.to_string());
            for token in &*server.rooms.get(&room_id)? {
                let out = server.senders.get(&token)?;
                println!("Sending \"{:?}\" to {:?}", msg, token);
                out.send(to_send.clone());
            }
        }
        Some(())
    }
    fn join_room(&mut self, str: String) -> Option<()> {
        println!("{:?} joining {}", self.id, str);
        let server = &mut *self.server.borrow_mut();
        let new_room = if server.names.contains_key(&str) {
            *server.names.get(&str)?
        } else {
            server.generate_room(str)
        };
        println!("1");
        if server.inRoom.contains_key(&self.id) {
            let room_id = *server.inRoom.get(&self.id)?;
            if room_id == new_room {
                return Some(());
            }
            server.rooms.get_mut(&room_id)?.remove(&self.id);
            server.inRoom.remove(&self.id);
        }
        println!("2");

        let room = server.rooms.entry(new_room).or_insert(HashSet::new());
        room.insert(self.id);
        println!("{} people in the room", room.len());
        server.inRoom.insert(self.id, new_room);
        println!("Success");
        Some(())
    }
    fn alias(&mut self, str: String) {
        let server = &mut *self.server.borrow_mut();
        if server.alias.contains_key(&self.id) {
            server.alias.remove(&self.id);
        }
        server.alias.insert(self.id, str);
    }
}

struct Server {
    senders: HashMap<Token, Sender>,
    names: HashMap<String, usize>,
    inRoom: HashMap<Token, usize>,
    alias: HashMap<Token, String>,
    rooms: HashMap<usize, HashSet<Token>>,
    to_generate: usize,
}

impl Server {
    fn new() -> Server {
        Server {
            senders: HashMap::new(),
            inRoom: HashMap::new(),
            rooms: HashMap::new(),
            alias: HashMap::new(),
            names: HashMap::new(),
            to_generate: 0,
        }
    }
    fn generate_room(&mut self, str: String) -> usize {
        self.names.insert(str, self.to_generate);
        self.rooms.insert(self.to_generate, HashSet::new());
        self.to_generate += 1;
        self.to_generate
    }
}

impl Handler for Client {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        // We have a new connection, so we increment the connection counter
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        // Tell the user the current count
        //        println!("The number of live connections is {}", self.senders.borrow().len());
        match msg {
            Message::Text(x) => {
                println!("{} from {:?}", x, self.id);
                if x.starts_with('#') {
                    self.join_room(x);
                } else if x.starts_with('@') {
                    self.alias(x);
                } else {
                    self.send_message(x);
                }
            }
            Message::Binary(_) => (),
        }

        // Echo the message back
/*        for (_,out) in & *self..borrow(){
          out.send(msg.clone());
        }*/
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away => println!("The client is leaving the site."),
            CloseCode::Abnormal => {
                println!("Closing handshake failed! Unable to obtain closing status from client.")
            }
            _ => println!("The client encountered an error: {}", reason),
        }
        let server = &mut *self.server.borrow_mut();

        if server.inRoom.contains_key(&self.id) {
            let room_id = *server.inRoom.get(&self.id).unwrap();
            server.rooms.get_mut(&room_id).unwrap().remove(&self.id);
            server.inRoom.remove(&self.id);
        }

        server.alias.remove(&self.id);
    }

    fn on_error(&mut self, err: ws::Error) {
        println!("The server encountered an error: {:?}", err);
    }
}

fn main() {
    let server = Rc::new(RefCell::new(Server::new()));

    listen("127.0.0.1:3012", |out| {
        let id = out.token();
        server.borrow_mut().senders.insert(id, out);
        Client {
            id: id,
            server: server.clone(),
        }
    }).unwrap()
}
