use std::fs::File;
use std::io::{Read, BufReader};
use crate::tcode::Tcode;

enum Btype{
    Bindirect = 0,  // indirect branch
    Btrap = 1,      // trap
    Bexception = 2, // exception
    Binterrupt = 3  // interrupt
}

struct Packet {
    tcode: Tcode,
    src: u16,
    sync: u8,
    b_type: u8,
    icnt: u16,
    f_addr: u64,
    u_addr: u64,
    tstamp: u64
}

pub fn read_packet(stream: &mut BufReader<File>) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = [0u8; 1];
    let bytes = stream.read_exact(&mut buf)?;
    println!("bytes: {:?}", bytes);
    let tcode = Tcode::from(buf[0]>>2);
    println!("tcode: {:?}", tcode);
    Ok(())
}