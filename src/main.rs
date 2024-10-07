extern crate clap;
extern crate object;
extern crate capstone;

mod packet;
mod tcode;
mod bcode;
use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter};
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

// FIXME: hacky way to get the offset operand, always the last one
fn compute_offset(insn: &Insn) -> i64 {
    let offset = insn.op_str().unwrap().split(",").last().unwrap();
    let offset_value: i64;
    // remove the leading space
    let offset = offset.trim();
    if offset.starts_with("-0x") {
        offset_value = i64::from_str_radix(&offset[3..], 16).unwrap() * -1;
    } else if offset.starts_with("0x") {
        offset_value = i64::from_str_radix(&offset[2..], 16).unwrap();
    } else if offset.starts_with("-") {
        offset_value = i64::from_str_radix(&offset[1..], 10).unwrap() * -1;
    } else {
        offset_value = i64::from_str_radix(&offset[0..], 10).unwrap();
    } 
    println!("offset_value: {}", offset_value);
    offset_value
}

// step pc by the length of the instruction if it's not a inferable jump
// if it is a inferable jump, then calculate the jump target
// if it is branch or uninferable, then panic [TODO]
fn step_pc(pc: u64, insn: &Insn) -> u64 {
    let opcode = insn.mnemonic();
    if matches!(opcode, Some("j" | "jal" | "c.j" | "c.jal")) {
        let offset = compute_offset(insn);
        (pc as i64 + offset) as u64
    } else {
        pc + insn.len() as u64
    }
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
    let mut pc_prev = pc;
    let mut prev_addr = packet.f_addr;
    let mut packet_count = 0;
    // main replay loop
    while let Ok(packet) = packet::read_packet(&mut trace_reader) {
        packet_count += 1;
        match packet.tcode {   
            Tcode::TcodeDbr => {
                println!("TcodeDbr: {:?}", packet);
                let mut curr_icnt = packet.icnt;
                while curr_icnt > 0 {
                    let insn = insn_map.get(&pc).unwrap();
                    println!("{}", insn);
                    dump_writer.write_all(format!("{}", insn).as_bytes())?;
                    dump_writer.write_all(b"\n")?;
                    pc_prev = pc;
                    pc = step_pc(pc, insn);
                    curr_icnt -= insn.len() as u16 >> 1;
                    println!("icnt: 0x{:x}", curr_icnt);
                }
                // this must be a branch, so just add the imm
                let branch_insn = insn_map.get(&pc_prev).unwrap();
                println!("branch_insn: {}", branch_insn);
                // Calculate the jump target
                let offset_value = compute_offset(branch_insn);
                pc = (pc_prev as i64 + offset_value) as u64;
            }
            Tcode::TcodeIbr => {
                println!("TcodeIbr: {:?}", packet);
                let mut curr_icnt = packet.icnt;
                while curr_icnt > 0 {
                    let insn = insn_map.get(&pc).unwrap();
                    println!("{}", insn);
                    dump_writer.write_all(format!("{}", insn).as_bytes())?;
                    dump_writer.write_all(b"\n")?;
                    pc = step_pc(pc, insn);
                    curr_icnt -= insn.len() as u16 >> 1;
                }
                pc = refund_addr(packet.u_addr ^ prev_addr);
                prev_addr = packet.u_addr ^ prev_addr;
            }
            Tcode::TcodeProgTraceCorr => {
                println!("TcodeProgTraceCorr: {:?}", packet);
                let mut curr_icnt = packet.icnt;
                while curr_icnt > 0 {
                    let insn = insn_map.get(&pc).unwrap();
                    println!("{}", insn);
                    dump_writer.write_all(format!("{}", insn).as_bytes())?;
                    dump_writer.write_all(b"\n")?;
                    pc = step_pc(pc, insn);
                    curr_icnt -= insn.len() as u16 >> 1;
                }
                println!("This is the last packet, breaking!");
                break;
            }
            _ => {
                println!("unhandled tcode: {:?}", packet);
            }
        }
    }

    println!("No more packets! packet_count: {}", packet_count);

    Ok(())
}