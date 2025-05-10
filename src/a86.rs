use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result, ensure};
use elf::{ElfBytes, endian::AnyEndian};

type Address = u64;

#[derive(Debug)]
pub enum Register {
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rbp,
    Rsp,
    Rsi,
    Rdi,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

#[derive(Debug)]
pub enum Arg {
    Register(Register),
    Offset(Register, u64),
    Literal(u64),
}

#[derive(Debug)]
pub enum DestinationArg {
    Register(Register),
    Offset(Register, u64),
}

#[derive(Debug)]
pub enum JmpDestination {
    Address(Address),
    Register(Register),
}

#[derive(Debug)]
pub enum Instruction {
    Add(Arg, Arg),
    Sub(Arg, Arg),
    Mov(DestinationArg, Arg),
    Jmp(JmpDestination),
}

#[derive(Debug)]
pub struct Program {
    entry_point: Address,
    instructions: BTreeMap<Address, Instruction>,
    address_to_symbols: HashMap<Address, HashSet<String>>,
    symbols_to_address: HashMap<String, Address>,
}

impl Program {
    pub fn from_elf_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let elf_bytes = fs::read(path).context("Failed to read ELF file")?;
        let elf_file =
            ElfBytes::<AnyEndian>::minimal_parse(&elf_bytes).context("Failed to parse ELF file")?;

        // Parse the text section as x86 instructions
        let (parsing_table, string_table) = elf_file
            .symbol_table()
            .context("Failed to parse symbol table in ELF file")?
            .context("ELF file did not have a symbol table")?;
        println!(
            "symbols: {:?}",
            parsing_table
                .iter()
                .map(|x| (
                    x.st_value,
                    string_table
                        .get(x.st_name as usize)
                        .context("Did not find name in string table")
                        .unwrap()
                ))
                .collect::<Vec<_>>()
        );
        let mut address_to_symbols: HashMap<Address, HashSet<String>> = HashMap::new();
        let mut symbols_to_address: HashMap<String, Address> = HashMap::new();
        for symbol in parsing_table {
            let identifier = string_table
                .get(
                    symbol
                        .st_name
                        .try_into()
                        .context("ELF file had symbol with too-large identifier")?,
                )
                .context("Failed to lookup name associated with symbol")?;

            let address: Address = symbol.st_value;

            symbols_to_address.insert(identifier.to_owned(), address);
            // TODO: fix this
            // address_to_symbols
            //     .entry(address)
            //     .or_insert(HashSet::new())
            //     .and_modify(|s| { s.insert(identifier.to_owned()); } );
        }

        // Get the text section
        let text_section = elf_file
            .section_header_by_name(".text")
            .context("Failed to parse section table in ELF file")?
            .context("ELF file did not have a .text segment")?;
        let (code_bytes, compression_header) = elf_file
            .section_data(&text_section)
            .context("Failed to extract code from .text section of ELF file")?;
        ensure!(
            compression_header.is_none(),
            "ELF file had compression header"
        );

        todo!()
    }
}
