mod a86;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use a86::Program;

#[derive(Parser)]
struct Args {
    program: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Construct an A86 program from the input program
    let program = Program::from_elf_file(&args.program)?;
    println!("Program: {:#x?}", program);



    Ok(())
}
