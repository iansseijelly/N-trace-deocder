extern crate clap;
extern crate elf;

use clap::Parser;
use elf::{ElfStream};
use elf::endian::AnyEndian;
use elf::to_str::{e_machine_to_human_str, e_osabi_to_string, e_type_to_human_str};

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

fn print_file_header(ehdr: &elf::file::FileHeader<AnyEndian>) {
    let e_type_str = match e_type_to_human_str(ehdr.e_type) {
        Some(s) => s.to_string(),
        None => format!("e_type({:#x})", ehdr.e_type),
    };

    let e_machine_str = match e_machine_to_human_str(ehdr.e_machine) {
        Some(s) => s.to_string(),
        None => format!("e_machine({:#x})", ehdr.e_machine),
    };

    println!("File Header:");
    println!("  Class: {:?}", ehdr.class);
    println!("  Endianness: {:?}", ehdr.endianness);
    println!("  Object Type: {e_type_str}");
    println!("  Arch: {e_machine_str}");
    println!("  OSABI: {}", e_osabi_to_string(ehdr.osabi));
    println!("  Entry point address: {:#x}", ehdr.e_entry);
    println!(
        "  Start of program headers: {:#x} (bytes into file)",
        ehdr.e_phoff
    );
    println!(
        "  Start of section headers: {:#x} (bytes into file)",
        ehdr.e_shoff
    );
    println!("  Flags: {:#x}", ehdr.e_flags);
    println!("  Size of this header: {:#x}", ehdr.e_ehsize);
    println!("  Size of program header: {:#x}", ehdr.e_phentsize);
    println!("  Number of program headers: {:#x}", ehdr.e_phnum);
    println!("  Size of section header: {:#x}", ehdr.e_shentsize);
    println!("  Number of section headers: {:#x}", ehdr.e_shnum);
    println!(
        "  Section headers string table section index: {}",
        ehdr.e_shstrndx
    );
}

fn main() {
    let args = Args::parse();

    let path = std::path::PathBuf::from(args.elf);
    let io = std::fs::File::open(path).expect("Could not open file");
    let elf_file = ElfStream::<AnyEndian, _>::open_stream(io).expect("Failed to open ELF stream");

    print_file_header(&elf_file.ehdr);
}