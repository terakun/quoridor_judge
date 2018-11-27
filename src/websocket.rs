extern crate ws;

use ws::{CloseCode, Factory, Handler, Message, Sender};
use std::net::TcpStream;

use std::io::{BufRead, BufReader};
use std::io::{BufWriter, Write};

pub struct Server {
    stream: TcpStream,
    out: Sender,
}

impl Handler for Server {
    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        let mut writer = BufWriter::new(&self.stream);
        if let Message::Text(txt) = msg {
            writer.write(txt.as_bytes()).unwrap();
            let _ = writer.flush();
        }
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away => println!("The client is leaving the site."),
            _ => println!("The client encountered an error: {}", reason),
        }
    }
}

pub struct MyFactory;

impl Factory for MyFactory {
    type Handler = Server;

    fn connection_made(&mut self, ws: Sender) -> Server {
        let stream = TcpStream::connect("157.82.205.48:8080").unwrap();
        Server {
            stream: stream,
            out: ws,
        }
    }
}
