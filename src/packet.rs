use std::fs::File;
use std::io::{Read, BufReader};
use crate::tcode::Tcode;
use crate::bcode::Btype;
const MDO_OFFSET: usize = 2;
const MDO_MASK: usize = 0xFC;

const MESO_MASK: usize = 0b11;
const MESO_IDLE:  usize = 0b00;
const MESO_EOF:   usize = 0b01;
const MESO_RES:   usize = 0b10;
const MESO_LAST:  usize = 0b11;


#[derive(Debug)]
enum Sync {
    ProgTraceSync = 5,
}

#[derive(Debug)]
enum Evcode {
    Disabled = 4,
}

#[derive(Debug)]
pub struct Packet {
    pub tcode: Tcode,
    src: u16,
    sync: u8,
    b_type: Btype,
    pub icnt: u16,
    pub f_addr: u64,
    pub u_addr: u64,
    pub tstamp: u64,
    pub evcode: u16
}

impl Packet {
    fn new() -> Packet {
        Packet {
            tcode: Tcode::TcodeNull,
            src: 0,
            sync: 0,
            b_type: Btype::Bindirect,
            icnt: 0,
            f_addr: 0,
            u_addr: 0,
            tstamp: 0,
            evcode: 0
        }
    }
}

fn read_u8(stream: &mut BufReader<File>) -> Result<u8, Box<dyn std::error::Error>> {
    let mut buf = [0u8; 1];
    stream.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_till_last(stream: &mut BufReader<File>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    loop {
        let byte = read_u8(stream)?;
        result.push(byte);
        if byte & MESO_MASK as u8 == MESO_LAST as u8 { break; }
    }
    Ok(result)
} 

fn read_till_eof(stream: &mut BufReader<File>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    loop {
        let byte = read_u8(stream)?;
        result.push(byte);
        if byte & MESO_MASK as u8 == MESO_EOF as u8 { break; }
    }
    Ok(result)
}

pub fn read_packet(stream: &mut BufReader<File>) -> Result<Packet, Box<dyn std::error::Error>> {
    let mut packet = Packet::new();
    let next_byte = read_u8(stream)?;
    let tcode: Tcode = Tcode::from(next_byte >> MDO_OFFSET);
    match tcode {
        Tcode::TcodeProgTraceSync => {
            let data = read_till_eof(stream)?;
            let sync = (data[0] & 0x3C) >> MDO_OFFSET;
            assert!(sync == Sync::ProgTraceSync as u8);
            // grab the top 2 bits from data[0] and everything else from all other data into one u64
            let mut f_addr : u64 = (data[0] as u64 & 0xC0) >> 6;
            for i in 1..data.len() {
                let data_byte = (data[i] as u64 & MDO_MASK as u64) >> MDO_OFFSET << 2; 
                f_addr = f_addr | (data_byte << ((i-1) * 6));
            }
            let data = read_till_last(stream)?;
            let mut timestamp : u64 = 0;
            for i in 0..data.len() {
                let data_byte = (data[i] as u64 & MDO_MASK as u64) >> MDO_OFFSET; 
                timestamp = timestamp | (data_byte << (i * 6));
            }
            // f_addr = refund_addr(f_addr);
            packet.tcode = tcode;
            packet.sync = sync;
            packet.f_addr = f_addr;
            packet.tstamp = timestamp;
        }
        Tcode::TcodeDbr => {
            let data = read_till_eof(stream)?;
            let mut icnt : u16 = 0;
            for i in 0..data.len() {
                let data_byte = (data[i] as u16 & MDO_MASK as u16) >> MDO_OFFSET; 
                icnt = icnt | (data_byte << (i * 6));
            }
            let data = read_till_last(stream)?;
            let mut timestamp : u64 = 0;
            for i in 0..data.len() {
                let data_byte = (data[i] as u64 & MDO_MASK as u64) >> MDO_OFFSET; 
                timestamp = timestamp | (data_byte << (i * 6));
            }
            packet.tcode = tcode;
            packet.icnt = icnt;
            packet.tstamp = timestamp;
        }
        Tcode::TcodeIbr => {
            let data = read_till_eof(stream)?;
            let b_type = (data[0] & 0xC) >> MDO_OFFSET;
            // grab the top 4 bits from data[0] and everything else from all other data into one u64
            let mut icnt : u16 = (data[0] as u16 & 0xF0) >> 4;
            for i in 1..data.len() {
                let data_byte = (data[i] as u16 & MDO_MASK as u16) >> MDO_OFFSET << 4;
                icnt = icnt | (data_byte << ((i-1) * 6));
            }
            let data = read_till_eof(stream)?;
            let mut u_addr : u64 = 0;
            for i in 0..data.len() {
                let data_byte = (data[i] as u64 & MDO_MASK as u64) >> MDO_OFFSET; 
                u_addr = u_addr | (data_byte << (i * 6));
                println!("u_addr: 0x{:x}", u_addr);
            }
            let data = read_till_last(stream)?;
            let mut timestamp : u64 = 0;
            for i in 0..data.len() {
                let data_byte = (data[i] as u64 & MDO_MASK as u64) >> MDO_OFFSET; 
                timestamp = timestamp | (data_byte << (i * 6));
            }
            packet.tcode = tcode;
            packet.b_type = Btype::from(b_type);
            packet.icnt = icnt;
            packet.u_addr = u_addr;
            packet.tstamp = timestamp;
        }
        Tcode::TcodeProgTraceCorr => {
            let data = read_till_eof(stream)?;
            let evcode = (data[0] & 0x3C) >> MDO_OFFSET;
            println!("evcode: {:?}", evcode);
            // assert!(evcode == Evcode::Disabled as u8);
            // grab the top 2 bits from data[0] and everything else from all other data into one u64
            let mut icnt : u16 = (data[0] as u16 & 0xC0) >> 6;
            for i in 1..data.len() {
                let data_byte = (data[i] as u16 & MDO_MASK as u16) >> MDO_OFFSET << 2; 
                icnt = icnt | (data_byte << ((i-1) * 6));
            }
            let data = read_till_last(stream)?;
            let mut timestamp : u64 = 0;
            for i in 0..data.len() {
                let data_byte = (data[i] as u64 & MDO_MASK as u64) >> MDO_OFFSET; 
                timestamp = timestamp | (data_byte << (i * 6));
            }
            packet.tcode = tcode;
            packet.icnt = icnt;
            packet.tstamp = timestamp;
        }
        _ => {
            println!("unknown tcode: {:?}", tcode);
        }
    }
    Ok(packet)
}