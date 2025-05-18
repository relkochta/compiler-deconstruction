mod a86;
mod decompiler;
mod loot;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use a86::Program;
use decompiler::parse;

#[derive(Parser)]
struct Args {
    program: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Construct an A86 program from the input program
    let a86_program = Program::from_elf_file(&args.program)?;
    //println!("Program: {:#x?}", program);

    // Decompile the program
    let loot_program = parse(&a86_program)?;
    println!("Decompiled Program:");
    // println!("{:#x?}", loot_program);
    println!("{}", loot_program);

    Ok(())
}
