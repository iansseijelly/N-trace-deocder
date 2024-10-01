extern crate clap;
extern crate object;
extern crate capstone;

mod packet;
mod tcode;
mod bcode;
use std::fs::File;
use std::io::{Read, BufReader, BufWriter};
use std::collections::HashMap;
use clap::Parser;
use capstone::prelude::*;
use capstone::arch::riscv::{ArchMode, ArchExtraMode};
use capstone::Insn;
use object::{Object, ObjectSection};
use tcode::Tcode;


#[derive(Parser)]
#[command(name = "trace-decoder", version = "0.1.0", about = "Decode trace files")]
struct Args {
    #[arg(short, long)]
    trace: String,
    #[arg(short, long)]
    elf: String,
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
    #[arg(short, long, default_value_t = String::from("trace.dump"))]
    dump: String,
}

fn refund_addr(addr: u64) -> u64 {
    addr << 1
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut elf_file = File::open(args.elf)?;
    let mut elf_buffer = Vec::new();
    elf_file.read_to_end(&mut elf_buffer)?;

    let elf = object::File::parse(&*elf_buffer)?;
    
    // assert this is for 64 bit RISC-V
    assert!(elf.architecture() == object::Architecture::Riscv64);

    // Find the .text section (where the executable code resides)
    let text_section = elf.section_by_name(".text").ok_or("No .text section found")?;
    let text_data = text_section.data()?;
    let entry_point = elf.entry();
    
    let cs = Capstone::new()
        .riscv()
        .mode(ArchMode::RiscV64)
        .extra_mode([ArchExtraMode::RiscVC].iter().copied())
        .detail(true)
        .build()?;

    let decoded_instructions = cs.disasm_all(&text_data, entry_point)?;
    println!("found {} instructions", decoded_instructions.len());

    // create a map of address to instruction 
    let mut insn_map : HashMap<u64, &Insn> = HashMap::new();
    for insn in decoded_instructions.as_ref() {
        insn_map.insert(insn.address(), insn);
    }

    let trace_file = File::open(args.trace)?;
    let mut trace_reader : BufReader<File> = BufReader::new(trace_file);

    let dump_file = File::create(args.dump)?;
    let mut dump_writer = BufWriter::new(dump_file);

    let packet = packet::read_packet(&mut trace_reader)?;
    println!("packet: {:?}", packet);
    let mut pc = refund_addr(packet.f_addr);
    let mut prev_addr = packet.f_addr;
    // main replay loop
    while let Ok(packet) = packet::read_packet(&mut trace_reader) {
        match packet.tcode {   
            Tcode::TcodeDbr => {
                let mut curr_icnt = packet.icnt;
                while curr_icnt > 0 {
                    let insn = insn_map.get(&pc).unwrap();
                    println!("0x{:x}: {}", pc, insn.mnemonic().unwrap());
                    pc = pc + insn.len() as u64;
                    curr_icnt -= insn.len() as u16 >> 1;
                }
                println!("pc: 0x{:x}", pc);
                println!("curr_icnt: {}", curr_icnt);
            }
            Tcode::TcodeIbr => {
                println!("TcodeIbr: {:?}", packet);
                let mut curr_icnt = packet.icnt;
                while curr_icnt > 0 {
                    println!("curr_icnt: {}", curr_icnt);
                    let insn = insn_map.get(&pc).unwrap();
                    println!("0x{:x}: {}", pc, insn.mnemonic().unwrap());
                    pc = pc + insn.len() as u64;
                    curr_icnt -= insn.len() as u16 >> 1;
                }
                pc = refund_addr(packet.u_addr ^ prev_addr);
                prev_addr = packet.u_addr ^ prev_addr;
            }
            _ => {
                println!("unhandled tcode: {:?}", packet);
            }
        }
    }
    

    Ok(())
}