use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write};
use std::io;

use std::time::Duration;
use std::sync::mpsc;

const H: usize = 5;
const W: usize = 5;
const PLAYER_NUM: usize = 2;

fn in_area((y, x): (usize, usize)) -> bool {
    0 <= y && y <= H && 0 <= x && x <= W
}

#[derive(Clone, PartialEq)]
enum Dir {
    Vertical,
    Horizontal,
    Empty,
}

impl Dir {
    fn parse(input: &String) -> Option<Self> {
        match input.as_ref() {
            "V" => Some(Dir::Vertical),
            "H" => Some(Dir::Horizontal),
            _ => None,
        }
    }
}

type Table = Vec<Vec<Dir>>;

struct Quoridor {
    table: Table,
    me: (usize, usize),
    op: (usize, usize),
}

impl Quoridor {
    fn new() -> Self {
        Quoridor {
            table: vec![vec![Dir::Empty; W - 1]; H - 1],
            me: (H - 1, W / 2),
            op: (0, W / 2),
        }
    }
    fn display(&self) -> String {
        let mut table: Vec<Vec<char>> = vec![vec![' '; 2 * W - 1]; 2 * H - 1];

        for i in 0..H - 1 {
            for j in 0..W - 1 {
                match self.table[i][j] {
                    Dir::Vertical => {
                        table[2 * i][2 * j + 1] = '|';
                        table[2 * (i + 1)][2 * j + 1] = '|';
                    }
                    Dir::Horizontal => {
                        table[2 * i + 1][2 * j] = '-';
                        table[2 * i + 1][2 * (j + 1)] = '-';
                    }
                    _ => {}
                }
            }
        }
        for i in 0..H - 1 {
            for j in 0..W - 1 {
                table[2 * i + 1][2 * j + 1] = '*';
            }
        }
        table[2 * self.me.0][2 * self.me.1] = 'P';
        table[2 * self.op.0][2 * self.op.1] = 'E';
        let mut s = String::new();
        s += &(0..(2 * W + 1)).map(|_| "#").collect::<String>();
        s += "\n";
        for i in 0..2 * H - 1 {
            let row: String = table[i].clone().into_iter().collect();
            s += &format!("#{}#\n", row);
        }
        s += &(0..(2 * W + 1)).map(|_| "#").collect::<String>();
        s += "\n";
        s
    }

    fn play(&mut self, com: &Command) -> Result<(), String> {
        match com {
            Command::Put(y, x, dir) => {
                if self.table[*y - 1][*x] != Dir::Empty {
                    return Err("Wall has already built".to_string());
                }
                self.table[*y - 1][*x] = dir.clone();
            }
            Command::Move(y, x) => {
                if !in_area((*y, *x)) {
                    return Err("Position is out of bounds".to_string());
                }
                self.me = (*y, *x);
            }
        }
        std::mem::swap(&mut self.me, &mut self.op);
        Ok(())
    }
}

enum Command {
    Move(usize, usize),
    Put(usize, usize, Dir),
}

impl Command {
    fn parse(input: &str) -> Option<Self> {
        let input_vec: Vec<&str> = input.trim().split_whitespace().collect();
        if input_vec.len() < 2 {
            return None;
        }
        let y = input_vec[0].parse::<usize>().unwrap();
        let x = input_vec[1].parse::<usize>().unwrap();
        if input_vec.len() != 3 {
            Some(Command::Move(y, x))
        } else {
            let dir = match Dir::parse(&input_vec[2].to_string()) {
                Some(dir) => dir,
                None => {
                    return None;
                }
            };
            Some(Command::Put(y, x, dir))
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
            if num >= PLAYER_NUM {
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
                let mut stream: &TcpStream = &mut self.streams[(from_id + 1) as usize % PLAYER_NUM];
                println!("{}", message.len());
                let command = Command::parse(&message).expect("parse error");
                /* ゲーム進行 */
                match self.game.play(&command) {
                    Ok(()) => {}
                    Err(e) => {
                        println!("{}", e);
                        return Ok(());
                    }
                }
                let result = self.game.display();
                println!("{}", result);
                /* 結果を相手に出力 */
                stream.write(&result.as_bytes())?;
            }
        }
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
