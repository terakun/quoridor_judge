extern crate bit_vec;
extern crate ws;

mod base64;
mod websocket;
use bit_vec::BitVec;
use base64::{append, bitvec_to_base64, from_u16, from_u8};
use ws::{CloseCode, Factory, Handler, Message, Sender};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write};
use std::io;
use std::env;

use std::time::Duration;
use std::sync::mpsc;

const WALL_LIMIT: usize = 10;
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
    last_move: Option<(usize, usize)>,
    turn_num: u16,
    white_wall_num: usize,
    black_wall_num: usize,
}

impl Quoridor {
    fn new() -> Self {
        Quoridor {
            table: vec![vec![None; W - 1]; H - 1],
            white: (H - 1, W / 2),
            black: (0, W / 2),
            is_white_turn: true,
            last_move: None,
            turn_num: 1,
            white_wall_num: 0,
            black_wall_num: 0,
        }
    }

    fn checkwalldir(&self, y: i8, x: i8, dir: Dir) -> bool {
        if !in_wall_area(y, x) {
            return false;
        }
        let (y, x) = (y as usize, x as usize);
        self.table[y][x] != None && self.table[y][x].unwrap().0 == dir
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
                if self.checkwalldir(y as i8, x as i8 - 1, Dir::Horizontal)
                    || self.checkwalldir(y as i8, x as i8 + 1, Dir::Horizontal)
                {
                    return Err("Wall has already built".to_string());
                }
            }
            Dir::Vertical => {
                if self.checkwalldir(y as i8 - 1, x as i8, Dir::Vertical)
                    || self.checkwalldir(y as i8 + 1, x as i8, Dir::Vertical)
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
            Command::Put(y, x, dir) => match self.settable(*y + 1, *x, *dir) {
                Ok(()) => {
                    if self.is_white_turn {
                        self.table[*y][*x] = Some((*dir, Colour::White));
                        self.white_wall_num += 1;
                    } else {
                        self.table[*y][*x] = Some((*dir, Colour::Black));
                        self.black_wall_num += 1;
                    }
                    self.last_move = Some((*y, *x));
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
                    self.last_move = None;
                }
                Err(e) => {
                    return Err(e);
                }
            },
        }
        self.is_white_turn = !self.is_white_turn;
        self.turn_num += 1;
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
        let x = input_vec[0].parse::<usize>().unwrap();
        let y = input_vec[1].parse::<usize>().unwrap();
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
    fn socketformat(&self) -> String {
        let mut output = String::new();
        let (me, op, me_num, op_num) = (
            self.game.white,
            self.game.black,
            self.game.white_wall_num,
            self.game.black_wall_num,
        );
        output += &format!(
            "{} {} {} {} {} {}\n",
            me.1,
            me.0,
            op.1,
            op.0,
            WALL_LIMIT - me_num,
            WALL_LIMIT - op_num
        );

        for rows in &self.game.table {
            for cell in rows {
                output += &format!(
                    "{} ",
                    match cell {
                        Some((Dir::Horizontal, _)) => 1,
                        Some((Dir::Vertical, _)) => 2,
                        None => 0,
                    }
                );
            }
            output += "\n";
        }
        output
    }

    fn viewformat(&self) -> String {
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
        append(&mut bv, from_u8(white_h_walls.len() as u8, 4));
        for pos in white_h_walls {
            append(&mut bv, from_u8(wall_to_u8(pos), 6));
        }
        append(&mut bv, from_u8(white_v_walls.len() as u8, 4));
        for pos in white_v_walls {
            append(&mut bv, from_u8(wall_to_u8(pos), 6));
        }
        append(&mut bv, from_u8(black_h_walls.len() as u8, 4));
        for pos in black_h_walls {
            append(&mut bv, from_u8(wall_to_u8(pos), 6));
        }
        append(&mut bv, from_u8(black_v_walls.len() as u8, 4));
        for pos in black_v_walls {
            append(&mut bv, from_u8(wall_to_u8(pos), 6));
        }
        bv.push(self.game.is_white_turn);
        if let Some((y, x)) = self.game.last_move {
            bv.push(true);
            append(&mut bv, from_u8(wall_to_u8((y, x)), 6));
        } else {
            bv.push(false);
        }
        append(&mut bv, from_u16(self.game.turn_num, 10));
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
                        let message: Vec<u8> = b.iter()
                            .take_while(|&c| *c != 13 && *c != 0)
                            .map(|c| *c)
                            .collect();

                        let message = String::from_utf8(message).unwrap();
                        let _ = tx.send((id, message));
                    }
                }
            });
            num += 1;
        }

        println!("ready");
        for (id, mut stream) in self.streams.iter().enumerate() {
            stream.write(&format!("{}\n", id).as_bytes())?;
        }

        let socketmsg = self.socketformat();
        self.streams[0].write(&socketmsg.as_bytes())?;
        loop {
            thread::sleep(Duration::from_micros(100));
            for (from_id, message) in rx.recv().iter() {
                println!("{:?}", message);
                let command = Command::parse(&message).expect("parse error");

                if let Err(e) = self.game.play(&command) {
                    println!("{}", e);
                    break;
                }

                let result = self.game.display();
                let socketmsg = self.socketformat();
                let sendmsg = self.viewformat();
                println!("{}", socketmsg);
                println!("{}", sendmsg);
                let _ = self.broadcaster.send(ws::Message::Text(sendmsg));

                let mut stream: &TcpStream = &mut self.streams[(from_id + 1) as usize % PLAYER_NUM];
                stream.write(&socketmsg.as_bytes())?;
            }
        }
    }
}
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{} [ip address]", args[0]);
        return;
    }
    let ip = args[1].clone();
    let wsport = 3012;
    let socketport = 8080;
    let factory = websocket::MyFactory;
    let websocket = ws::WebSocket::new(factory).unwrap();
    let broadcaster = websocket.broadcaster();
    {
        let ip = ip.clone();
        std::thread::spawn(move || {
            websocket.listen(&format!("{}:{}", ip, wsport)).unwrap();
        });
    }

    let mut server = JudgeServer {
        ip: format!("{}:{}", ip, socketport),
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
