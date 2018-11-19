use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write};
use std::io;

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::time::Duration;
use std::sync::mpsc;

const H: u8 = 5;
const W: u8 = 5;

struct Quoridor {
    table: Vec<Vec<bool>>,
    me: (u8, u8),
    op: (u8, u8),
}

impl Quoridor {
    fn new() -> Self {
        Quoridor {
            table: vec![vec![false; W as usize]; (2 * H - 1) as usize],
            me: (0, W / 2),
            op: (H - 1, W / 2),
        }
    }
    fn display(&self) -> String {
        let mut s = String::new();
        for i in 0..(2 * W + 1) {
            s += "#";
        }
        s += "\n";
        for i in 0..(2 * H - 1) {
            s += "#";
            for j in 0..W {
                if i % 2 == 0 {
                    if self.me == (i / 2, j) {
                        s += "A";
                    } else if self.op == (i / 2, j) {
                        s += "B";
                    } else {
                        s += " ";
                    }
                    if j != W - 1 {
                        if self.table[i as usize][j as usize] {
                            s += "|";
                        } else {
                            s += ".";
                        }
                    }
                } else {
                    if self.table[i as usize][j as usize] {
                        s += "-";
                    } else {
                        s += ".";
                    }
                    if j != W - 1 {
                        s += ".";
                    }
                }
            }
            s += "#";
            s += "\n";
        }
        for i in 0..(2 * W + 1) {
            s += "#";
        }
        s += "\n";
        s
    }
    fn play(&mut self, com: &Command) -> Result<(), String> {
        match com {
            Command::Put { p1, p2 } => {
                self.table[p1.0 as usize][p1.1 as usize] = true;
                self.table[p2.0 as usize][p2.1 as usize] = true;
            }
            dir => {
                let (dy, dx) = dir.to_dydx();
                self.op.0 = (self.op.0 as i8 + dy) as u8;
                self.op.1 = (self.op.1 as i8 + dx) as u8;
            }
        }
        // std::mem::swap(&mut self.me, &mut self.op);
        Ok(())
    }
}

/*
#######
# #b# #
#######
# # # #
#######
# #a# #
#######
*/
enum Command {
    Left,
    Up,
    Right,
    Down,
    Put { p1: (u8, u8), p2: (u8, u8) },
}

impl Command {
    fn to_dydx(&self) -> (i8, i8) {
        match *self {
            Command::Left => (0, -1),
            Command::Up => (-1, 0),
            Command::Right => (0, 1),
            Command::Down => (1, 0),
            _ => {
                panic!("error in to_dydx");
            }
        }
    }
    fn parse(input: &str) -> Self {
        match input {
            "L" => Command::Left,
            "U" => Command::Up,
            "R" => Command::Right,
            "D" => Command::Down,
            s => {
                let pos: Vec<u8> = s.trim()
                    .split_whitespace()
                    .map(|s| s.parse::<usize>().unwrap() as u8)
                    .collect();
                Command::Put {
                    p1: (pos[0], pos[1]),
                    p2: (pos[2], pos[3]),
                }
            }
        }
    }
}

enum Player {
    Human,
    AI(String),
}

struct JudgeServer {
    ip: String,
    streams: Vec<TcpStream>,
    players: Vec<Player>,
    game: Quoridor,
}

impl JudgeServer {
    fn start(&mut self) -> io::Result<()> {
        println!("{}", self.game.display());
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
            self.players.push(Player::AI(addr.to_string()));
            let tx = tx.clone();

            let _ = thread::spawn(move || -> io::Result<()> {
                let id = num;
                println!("{} came", addr);
                loop {
                    let mut b = [0; 128];
                    let n = stream.read(&mut b)?;
                    if n == 0 {
                        return Ok(());
                    } else {
                        let message: Vec<u8> =
                            b.iter().take_while(|&c| *c != 13).map(|c| *c).collect();

                        let message = String::from_utf8(message).unwrap();
                        let _ = tx.send((id, message));
                    }
                }
            });
            num += 1;
        }
        for (id, mut stream) in self.streams.iter().enumerate() {
            stream.write(&format!("{}\n", id).as_bytes())?;
        }
        self.streams[0].write(&self.game.display().as_bytes())?;
        loop {
            thread::sleep(Duration::from_micros(100));
            for (from_id, message) in rx.recv().iter() {
                for (id, mut stream) in self.streams.iter().enumerate() {
                    println!("{}", message.len());
                    let command = Command::parse(&message);
                    if id != *from_id {
                        /* ゲーム進行 */
                        match self.game.play(&command) {
                            Ok(()) => {}
                            Err(e) => {
                                println!("{}", e);
                                return Ok(());
                            }
                        }
                        let result = self.game.display();
                        /* 結果を相手に出力 */
                        stream.write(&result.as_bytes())?;
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
        players: Vec::new(),
        game: Quoridor::new(),
    };
    match server.start() {
        Ok(_) => (),
        Err(e) => println!("{:?}", e),
    }
}
