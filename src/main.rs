extern crate bit_vec;
extern crate ws;

use bit_vec::BitVec;
use ws::{CloseCode, Factory, Handler, Message, Sender};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write};
use std::io;

use std::time::Duration;
use std::sync::mpsc;

fn from_u8(n: u8, len: usize) -> BitVec {
    let mut b = BitVec::new();
    let mlb = (1 << (len - 1)) as u8;
    for i in 0..len {
        let mask = (mlb >> i) as u8;
        b.push((n & mask) != 0);
    }
    b
}

fn u8_to_char(n: u8) -> char {
    match n {
        n if (0 <= n && n < 26) => ('A' as u8 + n) as char,
        n if (26 <= n && n < 52) => ('a' as u8 + (n - 26)) as char,
        n if (52 <= n && n < 62) => ('0' as u8 + (n - 52)) as char,
        62 => '+',
        63 => '/',
        _ => {
            panic!("something wrong:{}", n);
        }
    }
}

fn append(src: &mut BitVec, dst: BitVec) {
    for b in dst.iter() {
        src.push(b);
    }
}

fn bitvec_to_base64(mut bv: BitVec) -> String {
    let mut charv = Vec::new();
    let mut n = 0;
    for (i, b) in bv.iter().enumerate() {
        if i != 0 && i % 6 == 0 {
            let c = u8_to_char(n);
            charv.push(c);
            n = 0;
        }
        if b {
            n = n * 2 + 1;
        } else {
            n = n * 2;
        }
    }
    charv.iter().collect()
}

const H: usize = 9;
const W: usize = 9;
const PLAYER_NUM: usize = 2;
const DPOS: [(i8, i8); 4] = [(-1, 0), (0, -1), (1, 0), (0, 1)];

fn pos_to_u8((y, x): (usize, usize)) -> u8 {
    ((W - 1 - y) * W + x) as u8
}
fn wall_to_u8((y, x): (usize, usize)) -> u8 {
    ((W - 2 - y) * (W - 1) + x) as u8
}

fn in_area(y: usize, x: usize) -> bool {
    y < H && x < W
}
fn in_wall_area(y: i8, x: i8) -> bool {
    0 <= y && y < (H - 1) as i8 && 0 <= x && x < (W - 1) as i8
}

#[derive(Clone, Copy, PartialEq)]
enum Colour {
    White,
    Black,
}

#[derive(Clone, Copy, PartialEq)]
enum Dir {
    Vertical,
    Horizontal,
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

type Table = Vec<Vec<Option<(Dir, Colour)>>>;

struct Quoridor {
    table: Table,
    white: (usize, usize),
    black: (usize, usize),
    is_white_turn: bool,
}

impl Quoridor {
    fn new() -> Self {
        Quoridor {
            table: vec![vec![None; W - 1]; H - 1],
            white: (H - 1, W / 2),
            black: (0, W / 2),
            is_white_turn: true,
        }
    }

    fn settable(&self, y: usize, x: usize, dir: Dir) -> Result<(), String> {
        let y = y - 1;
        if !in_wall_area(y as i8, x as i8) {
            return Err("Put position is out of bounds".to_string());
        }
        if self.table[y][x] != None {
            return Err("Wall has already built".to_string());
        }
        match dir {
            Dir::Horizontal => {
                if in_wall_area(y as i8, x as i8 - 1)
                    && (self.table[y][x - 1] != None
                        && self.table[y][x - 1].unwrap().0 == Dir::Horizontal)
                {
                    return Err("Wall has already built".to_string());
                }
            }
            Dir::Vertical => {
                if in_wall_area(y as i8 - 1, x as i8)
                    && (self.table[y - 1][x] != None
                        && self.table[y - 1][x].unwrap().0 == Dir::Vertical)
                {
                    return Err("Wall has already built".to_string());
                }
            }
        }
        Ok(())
    }
    fn exist_wall(&self, y: i8, x: i8, dy: i8, dx: i8) -> bool {
        let (y1, x1, y2, x2, dir) = if dx != 0 {
            if dx == 1 {
                (y - 1, x, y, x, Dir::Vertical)
            } else {
                (y - 1, x, y, x - 1, Dir::Vertical)
            }
        } else {
            if dy == 1 {
                (y, x, y, x - 1, Dir::Horizontal)
            } else {
                (y - 1, x, y - 1, x - 1, Dir::Horizontal)
            }
        };
        let cell1 = if in_wall_area(y1, x1) {
            self.table[y1 as usize][x1 as usize]
        } else {
            None
        };
        let cell2 = if in_wall_area(y2, x2) {
            self.table[y2 as usize][x2 as usize]
        } else {
            None
        };
        (cell1 != None && cell1.unwrap().0 == dir) || (cell2 != None && cell2.unwrap().0 == dir)
    }
    fn next_moves(&self) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        let (me, op) = if self.is_white_turn {
            (
                (self.white.0 as i8, self.white.1 as i8),
                (self.black.0 as i8, self.black.1 as i8),
            )
        } else {
            (
                (self.black.0 as i8, self.black.1 as i8),
                (self.white.0 as i8, self.white.1 as i8),
            )
        };
        for (dy, dx) in DPOS.iter() {
            if self.exist_wall(me.0, me.1, *dy, *dx) {
                continue;
            }
            let (y, x) = (me.0 + dy, me.1 + dx);

            if (y, x) == op {
                for (dy, dx) in DPOS.iter() {
                    if self.exist_wall(y, x, *dy, *dx) {
                        break;
                    }
                    let (y2, x2) = (y + dy, x + dx);
                    if me == (y2, x2) {
                        continue;
                    }
                    moves.push((y2 as usize, x2 as usize));
                }
            } else {
                moves.push((y as usize, x as usize));
            }
        }
        moves
    }
    fn movable(&self, y: usize, x: usize) -> Result<(), String> {
        if !in_area(y, x) {
            return Err("Position is out of bounds".to_string());
        }
        let moves = self.next_moves();
        println!("{:?}", moves);
        for m in moves {
            if (y, x) == m {
                return Ok(());
            }
        }
        Err("illegal operation".to_string())
    }
    fn display(&self) -> String {
        let mut table: Vec<Vec<char>> = vec![vec![' '; 2 * W - 1]; 2 * H - 1];

        for i in 0..H - 1 {
            for j in 0..W - 1 {
                match self.table[i][j] {
                    Some((Dir::Vertical, _)) => {
                        table[2 * i][2 * j + 1] = '|';
                        table[2 * (i + 1)][2 * j + 1] = '|';
                    }
                    Some((Dir::Horizontal, _)) => {
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
        table[2 * self.white.0][2 * self.white.1] = 'W';
        table[2 * self.black.0][2 * self.black.1] = 'B';
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
            Command::Put(y, x, dir) => match self.settable(*y, *x, *dir) {
                Ok(()) => {
                    if self.is_white_turn {
                        self.table[*y - 1][*x] = Some((*dir, Colour::White));
                    } else {
                        self.table[*y - 1][*x] = Some((*dir, Colour::Black));
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            },
            Command::Move(y, x) => match self.movable(*y, *x) {
                Ok(()) => {
                    if self.is_white_turn {
                        self.white = (*y, *x);
                    } else {
                        self.black = (*y, *x);
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            },
        }
        self.is_white_turn = !self.is_white_turn;
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
        if input_vec.len() < 3 {
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
    broadcaster: Sender,
}

impl JudgeServer {
    fn gameformat(&self) -> String {
        let mut bv = BitVec::new();
        bv.push(true);
        bv.push(false);
        append(&mut bv, from_u8(pos_to_u8(self.game.white), 7));
        append(&mut bv, from_u8(pos_to_u8(self.game.black), 7));
        let mut white_h_walls = Vec::new();
        let mut white_v_walls = Vec::new();
        let mut black_h_walls = Vec::new();
        let mut black_v_walls = Vec::new();
        for (y, row) in self.game.table.iter().enumerate() {
            for (x, wall) in row.iter().enumerate() {
                match wall {
                    Some((Dir::Horizontal, Colour::White)) => {
                        white_h_walls.push((y, x));
                    }
                    Some((Dir::Vertical, Colour::White)) => {
                        white_v_walls.push((y, x));
                    }
                    Some((Dir::Horizontal, Colour::Black)) => {
                        black_h_walls.push((y, x));
                    }
                    Some((Dir::Vertical, Colour::Black)) => {
                        black_v_walls.push((y, x));
                    }
                    None => {}
                }
            }
        }
        println!("{}", white_h_walls.len());
        append(&mut bv, from_u8(white_h_walls.len() as u8, 4));
        for pos in white_h_walls {
            append(&mut bv, from_u8(wall_to_u8(pos), 6));
        }
        println!("{}", white_v_walls.len());
        append(&mut bv, from_u8(white_v_walls.len() as u8, 4));
        for pos in white_v_walls {
            append(&mut bv, from_u8(wall_to_u8(pos), 6));
        }
        println!("{}", black_h_walls.len());
        append(&mut bv, from_u8(black_h_walls.len() as u8, 4));
        for pos in black_h_walls {
            append(&mut bv, from_u8(wall_to_u8(pos), 6));
        }
        println!("{}", black_v_walls.len());
        append(&mut bv, from_u8(black_v_walls.len() as u8, 4));
        for pos in black_v_walls {
            append(&mut bv, from_u8(wall_to_u8(pos), 6));
        }
        bitvec_to_base64(bv)
    }

    fn start(&mut self) -> io::Result<()> {
        println!("{}", self.game.display());

        let lis = TcpListener::bind(&self.ip)?;
        let mut num = 0;
        let (tx, rx) = mpsc::channel();

        while num < PLAYER_NUM {
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
                let command = Command::parse(&message).expect("parse error");

                if let Err(e) = self.game.play(&command) {
                    println!("{}", e);
                    break;
                }

                let result = self.game.display();
                let sendmsg = self.gameformat();
                println!("{}", result);
                let _ = self.broadcaster.send(ws::Message::Text(sendmsg));

                let mut stream: &TcpStream = &mut self.streams[(from_id + 1) as usize % PLAYER_NUM];
                stream.write(&result.as_bytes())?;
            }
        }
    }
}

struct Server {
    out: Sender,
}

impl Handler for Server {
    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        self.out.send(msg)
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away => println!("The client is leaving the site."),
            _ => println!("The client encountered an error: {}", reason),
        }
    }
}

struct MyFactory;

impl Factory for MyFactory {
    type Handler = Server;

    fn connection_made(&mut self, ws: Sender) -> Server {
        Server { out: ws }
    }
}

fn main() {
    let factory = MyFactory;
    let websocket = ws::WebSocket::new(factory).unwrap();
    let broadcaster = websocket.broadcaster();
    std::thread::spawn(|| {
        websocket.listen("127.0.0.1:3012").unwrap();
    });

    let mut server = JudgeServer {
        ip: "127.0.0.1:8080".to_string(),
        streams: Vec::new(),
        players: Vec::new(),
        game: Quoridor::new(),
        broadcaster: broadcaster,
    };
    match server.start() {
        Ok(_) => (),
        Err(e) => println!("{:?}", e),
    }
}
