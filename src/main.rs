use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write};
use std::io;

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::time::Duration;
use std::sync::mpsc;

// struct Quoridor {
// table: [[; 9],
// }

enum Player {
    Human,
    AI(String),
}

struct JudgeServer {
    ip: String,
    streams: Vec<TcpStream>,
    // players: (Option<Player>, Option<Player>),
    // game: Quoridor,
}

impl JudgeServer {
    fn start(&mut self) -> io::Result<()> {
        let lis = TcpListener::bind(&self.ip)?;
        let mut num = 0;
        let (tx, rx) = mpsc::channel();
        loop {
            if num >= 2 {
                break;
            }
            let (mut stream, addr) = match lis.accept() {
                Ok(result) => result,
                Err(e) => {
                    println!("couldn't get client: {:?}", e);
                    break;
                }
            };
            self.streams.push(stream.try_clone().unwrap());
            let tx = tx.clone();
            let _ = thread::spawn(move || -> io::Result<()> {
                let id = num;
                println!("{} came", addr);
                loop {
                    let mut b = [0; 1024];
                    let n = stream.read(&mut b)?;

                    if n == 0 {
                        continue;
                    } else {
                        let message = std::str::from_utf8(&b).unwrap().to_string();
                        let _ = tx.send((id, message));
                    }
                }
            });
            num += 1;
        }
        loop {
            thread::sleep(Duration::from_micros(100));
            for (from_id, message) in rx.recv().iter() {
                println!("{}:{}", from_id, message);
                for (id, mut stream) in self.streams.iter().enumerate() {
                    if id != *from_id {
                        stream.write(&message.as_bytes())?;
                    }
                }
            }
        }
        Ok(())
    }
}
fn main() {
    let mut server = JudgeServer {
        ip: "127.0.0.1:8080".to_string(),
        streams: Vec::new(),
    };
    match server.start() {
        Ok(_) => (),
        Err(e) => println!("{:?}", e),
    }
}
