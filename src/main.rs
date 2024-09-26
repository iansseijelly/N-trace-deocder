extern crate clap;
extern crate object;
extern crate capstone;

mod packet;
mod tcode;

use std::fs::File;
use std::io::{Read, BufReader};
use std::collections::HashMap;
use clap::Parser;
use capstone::prelude::*;
use capstone::arch::riscv::{ArchMode, ArchExtraMode};
use capstone::Insn;
use object::{Object, ObjectSection};


#[derive(Parser)]
#[command(name = "trace-decoder", version = "0.1.0", about = "Decode trace files")]
struct Args {
    #[arg(short, long)]
    trace: String,
    #[arg(short, long)]
    elf: String,
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
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

    let packet = packet::read_packet(&mut trace_reader)?;
    println!("decoded packet: {:?}", packet);
    while let Ok(packet) = packet::read_packet(&mut trace_reader) {
        println!("decoded packet: {:?}", packet);
    }
    Ok(())
}