use std::fs::File;
use std::io::{Read, BufReader};
use crate::tcode::Tcode;

const MDO_OFFSET: usize = 2;
const MDO_MASK: usize = 0xFC;

const MESO_MASK: usize = 0b11;
const MESO_IDLE:  usize = 0b00;
const MESO_EOF:   usize = 0b01;
const MESO_RES:   usize = 0b10;
const MESO_LAST:  usize = 0b11;

enum Btype{
    Bindirect = 0,  // indirect branch
    Btrap = 1,      // trap
    Bexception = 2, // exception
    Binterrupt = 3  // interrupt
}

enum Sync {
    ProgTraceSync = 5,
}

struct Packet {
    tcode: Tcode,
    src: u16,
    sync: u8,
    b_type: Btype,
    icnt: u16,
    f_addr: u64,
    u_addr: u64,
    tstamp: u64
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

fn refund_addr(addr: u64) -> u64 {
    addr << 1
}

pub fn read_packet(stream: &mut BufReader<File>) -> Result<(), Box<dyn std::error::Error>> {
    let next_byte = read_u8(stream)?;
    let tcode = Tcode::from(next_byte>>MDO_OFFSET);
    match tcode {
        Tcode::TcodeProgTraceSync => {
            println!("prog trace sync");
            let data = read_till_last(stream)?;
            let sync = (data[0] & 0x3C) >> MDO_OFFSET;
            assert!(sync == Sync::ProgTraceSync as u8);
            // grab the top 2 bits from data[0] and everything else from all other data into one u64
            let mut f_addr : u64 = (data[0] as u64 & 0xC0) >> 6;
            println!("f_addr: 0x{:016x}", f_addr);
            println!("data[0]: {:02x}", data[0]);
            for i in 1..data.len() {
                println!("f_addr: 0x{:016x}", f_addr);
                let data_byte = (data[i] as u64 & MDO_MASK as u64) >> MDO_OFFSET << 2; 
                println!("data_byte: 0x{:016x}", data_byte);
                f_addr = f_addr | (data_byte << ((i-1) * 6));
            }
            f_addr = refund_addr(f_addr);
            println!("f_addr: 0x{:016x}", f_addr);
        }
        Tcode::TcodeDbr => {
            println!("dbr");
        }
        Tcode::TcodeIbr => {
            println!("ibr");
        }
        _ => {
            println!("unknown tcode: {:?}", tcode);
        }
    }
    Ok(())
}