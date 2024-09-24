extern crate clap;
extern crate object;
extern crate capstone;

use std::fs::File;
use std::io::Read;
use clap::Parser;
use capstone::prelude::*;
use capstone::arch::riscv::{ArchMode, ArchExtraMode};
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

fn parse_elf(file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(file_name)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let elf = object::File::parse(&*buffer)?;
    
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

    let decoded_instructions = cs.disasm_all(&text_data, entry_point).unwrap();
    println!("found {} instructions", decoded_instructions.len());

    Ok(())
}

fn main() {
    let args = Args::parse();

    let mut file = File::open(args.elf.clone()).expect("Failed to open ELF file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read ELF file");

    parse_elf(&args.elf).expect("Failed to parse ELF file");

}