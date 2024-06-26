use std::collections::HashSet;
use serde::{Deserialize, Serialize};

use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub addr: Recipient<Message>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub msg: String,
}

#[derive(Message, Debug)]
#[derive(Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct BME280Measurement {
    pub humidity: f32,
    pub temperature: f32,
    pub pressure: f32,
}

#[derive(Debug)]
pub struct ControlHub {
    sessions: HashSet<Recipient<Message>>,
}

impl ControlHub {
    pub fn new() -> ControlHub {
        ControlHub {
            sessions: HashSet::new(),
        }
    }
}

impl ControlHub {
    fn send_message(&self, message: &str) {
        for addr in self.sessions.iter() {
            addr.do_send(Message(message.to_owned()));
        }
    }
}

impl Actor for ControlHub {
    type Context = Context<Self>;
}

impl Handler<Connect> for ControlHub {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");
        self.sessions.insert(msg.addr).into()
    }
}

impl Handler<Disconnect> for ControlHub {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");
        self.sessions.remove(&msg.addr);
    }
}

impl Handler<BME280Measurement> for ControlHub {
    type Result = ();

    fn handle(&mut self, msg: BME280Measurement, _: &mut Context<Self>) {
        self.send_message(serde_json::to_string(&msg).unwrap().as_str());
    }
}

impl Handler<ClientMessage> for ControlHub {
    type Result = ();

    fn handle(&mut self, _: ClientMessage, _: &mut Context<Self>) {
    }
}
