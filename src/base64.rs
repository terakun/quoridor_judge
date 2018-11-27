extern crate bit_vec;

use self::bit_vec::BitVec;
pub fn from_u8(n: u8, len: usize) -> BitVec {
    let mut b = BitVec::new();
    let mlb = (1 << (len - 1)) as u8;
    for i in 0..len {
        let mask = (mlb >> i) as u8;
        b.push((n & mask) != 0);
    }
    b
}

pub fn from_u16(n: u16, len: usize) -> BitVec {
    let mut b = BitVec::new();
    let mlb = (1 << (len - 1)) as u16;
    for i in 0..len {
        let mask = (mlb >> i) as u16;
        b.push((n & mask) != 0);
    }
    b
}

fn u8_to_char(n: u8) -> char {
    match n {
        n if n < 26 => ('A' as u8 + n) as char,
        n if (26 <= n && n < 52) => ('a' as u8 + (n - 26)) as char,
        n if (52 <= n && n < 62) => ('0' as u8 + (n - 52)) as char,
        62 => '+',
        63 => '/',
        _ => {
            panic!("something wrong:{}", n);
        }
    }
}

pub fn append(src: &mut BitVec, dst: BitVec) {
    for b in dst.iter() {
        src.push(b);
    }
}

pub fn bitvec_to_base64(mut bv: BitVec) -> String {
    let mut charv = Vec::new();
    let mut n = 0;
    while bv.len() % 6 != 0 {
        bv.push(false);
    }
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
    let c = u8_to_char(n);
    charv.push(c);
    charv.iter().collect()
}
