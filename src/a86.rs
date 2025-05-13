use std::{
    collections::{HashMap, HashSet},
    fs, iter,
    path::Path,
};

use anyhow::{Context, Result, bail, ensure};
use bimap::BiMap;
use elf::{ElfBytes, endian::AnyEndian};
use iced_x86::{Code, Decoder, DecoderOptions, OpKind};

type Address = u64;

#[derive(Debug, Clone, Copy)]
pub enum Register {
    // 32-bit registers
    Eax,
    R9d,
    // 64-bit registers
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

impl TryFrom<iced_x86::Register> for Register {
    type Error = anyhow::Error;

    fn try_from(value: iced_x86::Register) -> std::result::Result<Self, Self::Error> {
        use Register as r;
        use iced_x86::Register as doodoo;
        Ok(match value {
            doodoo::EAX => r::Eax,
            doodoo::R9D => r::R9d,
            doodoo::RAX => r::Rax,
            doodoo::RBX => r::Rbx,
            doodoo::RCX => r::Rcx,
            doodoo::RDX => r::Rdx,
            doodoo::RBP => r::Rbp,
            doodoo::RSP => r::Rsp,
            doodoo::RSI => r::Rsi,
            doodoo::RDI => r::Rdi,
            doodoo::R8 => r::R8,
            doodoo::R9 => r::R9,
            doodoo::R10 => r::R10,
            doodoo::R11 => r::R11,
            doodoo::R12 => r::R12,
            doodoo::R13 => r::R13,
            doodoo::R14 => r::R14,
            doodoo::R15 => r::R15,
            _ => bail!("doodoo {:?}", value),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Arg {
    Address(Address),
    Register(Register),
    Offset(Register, i64),
    Literal(u64),
}

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Add(Arg, Arg),
    Sub(Arg, Arg),
    And(Arg, Arg),
    Xor(Arg, Arg),
    Mov(Arg, Arg),
    Cmove(Arg, Arg),
    Cmovl(Arg, Arg),
    Cmp(Arg, Arg),
    Call(Address),
    Jmp(Arg),
    Jne(Arg),
    Je(Arg),
    Jl(Arg),
    Jg(Arg),
    Push(Arg),
    Pop(Arg),
    Lea(Arg, Arg),
    Ret,
}

impl TryFrom<iced_x86::Instruction> for Instruction {
    type Error = anyhow::Error;

    fn try_from(value: iced_x86::Instruction) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            t if t.code() == Code::Push_r64 => {
                Instruction::Push(Arg::Register(t.op_register(0).try_into()?))
            }
            t if t.code() == Code::Pop_r64 => {
                Instruction::Pop(Arg::Register(t.op_register(0).try_into()?))
            }
            t if t.code() == Code::Jmp_rm64 => {
                Instruction::Jmp(Arg::Register(t.op_register(0).try_into()?))
            }
            t if t.code() == Code::Jmp_rel8_64 => {
                Instruction::Jmp(Arg::Address(t.memory_displacement64()))
            }
            t if t.code() == Code::Je_rel32_64 || t.code() == Code::Je_rel8_64 => {
                Instruction::Je(Arg::Address(t.memory_displacement64()))
            }
            t if t.code() == Code::Jl_rel32_64 || t.code() == Code::Jl_rel8_64 => {
                Instruction::Jl(Arg::Address(t.memory_displacement64()))
            }
            t if t.code() == Code::Jg_rel32_64 || t.code() == Code::Jg_rel8_64 => {
                Instruction::Jg(Arg::Address(t.memory_displacement64()))
            }
            t if t.code() == Code::Jne_rel32_64 || t.code() == Code::Jne_rel8_64 => {
                Instruction::Jne(Arg::Address(t.memory_displacement64()))
            }
            t if t.code() == Code::Call_rel32_64 => Instruction::Call(t.memory_displacement64()),
            t if t.code() == Code::Cmp_rm64_imm8 => Instruction::Cmp(
                Arg::Register(t.op_register(0).try_into()?),
                Arg::Literal(t.immediate(1)),
            ),
            t if t.code() == Code::Add_rm64_imm8
                || t.code() == Code::Add_rm64_imm32
                || t.code() == Code::Add_rm32_imm8
                || t.code() == Code::Add_rm32_imm32 =>
            {
                Instruction::Add(
                    Arg::Register(t.op_register(0).try_into()?),
                    Arg::Literal(t.immediate(1)),
                )
            }
            t if t.code() == Code::Add_rm64_r64 || t.code() == Code::Add_rm32_r32 => {
                Instruction::Add(
                    Arg::Register(t.op_register(0).try_into()?),
                    Arg::Register(t.op_register(1).try_into()?),
                )
            }
            t if t.code() == Code::Sub_rm64_imm8
                || t.code() == Code::Sub_rm64_imm32
                || t.code() == Code::Sub_rm32_imm8
                || t.code() == Code::Sub_rm32_imm32 =>
            {
                Instruction::Sub(
                    Arg::Register(t.op_register(0).try_into()?),
                    Arg::Literal(t.immediate(1)),
                )
            }
            t if t.code() == Code::Sub_rm64_r64 || t.code() == Code::Sub_rm32_r32 => {
                Instruction::Sub(
                    Arg::Register(t.op_register(0).try_into()?),
                    Arg::Register(t.op_register(1).try_into()?),
                )
            }
            t if t.code() == Code::And_rm64_imm8
                || t.code() == Code::And_rm64_imm32
                || t.code() == Code::And_rm32_imm8
                || t.code() == Code::And_rm32_imm32 =>
            {
                Instruction::And(
                    Arg::Register(t.op_register(0).try_into()?),
                    Arg::Literal(t.immediate(1)),
                )
            }
            t if t.code() == Code::And_rm64_r64 || t.code() == Code::And_rm32_r32 => {
                Instruction::And(
                    Arg::Register(t.op_register(0).try_into()?),
                    Arg::Register(t.op_register(1).try_into()?),
                )
            }
            t if t.code() == Code::Xor_rm64_imm8
                || t.code() == Code::Xor_rm64_imm32
                || t.code() == Code::Xor_rm32_imm8
                || t.code() == Code::Xor_rm32_imm32 =>
            {
                Instruction::Xor(
                    Arg::Register(t.op_register(0).try_into()?),
                    Arg::Literal(t.immediate(1)),
                )
            }
            t if t.code() == Code::Xor_rm64_r64 || t.code() == Code::Xor_rm32_r32 => {
                Instruction::Xor(
                    Arg::Register(t.op_register(0).try_into()?),
                    Arg::Register(t.op_register(1).try_into()?),
                )
            }
            t if t.code() == Code::Mov_rm64_r64
                || t.code() == Code::Mov_r64_rm64
                || t.code() == Code::Mov_r32_rm32
                || t.code() == Code::Mov_rm32_r32 =>
            {
                Instruction::Mov(
                    match t.op0_kind() {
                        OpKind::Memory => Arg::Offset(
                            t.memory_base().try_into()?,
                            t.memory_displacement64() as i64,
                        ),
                        OpKind::Register => Arg::Register(t.op_register(0).try_into()?),
                        _ => bail!("incorrect operand kind found for move dst"),
                    },
                    match t.op1_kind() {
                        OpKind::Memory => Arg::Offset(
                            t.memory_base().try_into()?,
                            t.memory_displacement64() as i64,
                        ),
                        OpKind::Register => Arg::Register(t.op_register(1).try_into()?),
                        _ => bail!("incorrect operand kind found for move src"),
                    },
                )
            }
            t if t.code() == Code::Mov_r32_imm32 => Instruction::Mov(
                Arg::Register(t.op_register(0).try_into()?),
                Arg::Literal(t.immediate(1)),
            ),
            t if t.code() == Code::Cmove_r64_rm64 => Instruction::Cmove(
                Arg::Register(t.op_register(0).try_into()?),
                Arg::Register(t.op_register(1).try_into()?),
            ),
            t if t.code() == Code::Cmovl_r64_rm64 => Instruction::Cmovl(
                Arg::Register(t.op_register(0).try_into()?),
                Arg::Register(t.op_register(1).try_into()?),
            ),
            t if t.code() == Code::Lea_r64_m => Instruction::Lea(
                Arg::Register(t.op_register(0).try_into()?),
                Arg::Address(t.memory_displacement64()),
            ),
            t if t.code() == Code::Retnq => Instruction::Ret,
            t => bail!("instruction code {:#?} not implmented", t.code()),
        })
    }
}

#[derive(Debug)]
pub struct Program {
    /// The address of the instruction to which the "entry" symbol points
    entry_point: Address,

    /// All of the instructions in the program, in order
    instructions: Vec<Instruction>,
    /// A mapping between instructions' memory addresses
    /// and their index in the `instructions` vector
    memory_map: BiMap<Address, usize>,

    address_to_symbols: HashMap<Address, HashSet<String>>,
    symbols_to_address: HashMap<String, Address>,
}

impl Program {
    pub fn entry_point(&self) -> Address {
        self.entry_point
    }

    pub fn instructions(&self) -> &[Instruction] {
        self.instructions.as_slice()
    }

    pub fn address_to_index(&self, address: Address) -> Option<usize> {
        self.memory_map.get_by_left(&address).copied()
    }

    pub fn index_to_address(&self, index: usize) -> Option<Address> {
        self.memory_map.get_by_right(&index).copied()
    }

    pub fn address_to_symbols(&self, address: Address) -> HashSet<String> {
        self.address_to_symbols
            .get(&address)
            .cloned()
            .unwrap_or(HashSet::new())
    }

    pub fn symbol_to_address(&self, symbol: &str) -> Option<Address> {
        self.symbols_to_address.get(symbol).copied()
    }

    pub fn from_elf_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let elf_bytes = fs::read(path).context("Failed to read ELF file")?;
        let elf_file =
            ElfBytes::<AnyEndian>::minimal_parse(&elf_bytes).context("Failed to parse ELF file")?;

        // Construct the symbol table
        let (parsing_table, string_table) = elf_file
            .symbol_table()
            .context("Failed to parse symbol table in ELF file")?
            .context("ELF file did not have a symbol table")?;
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
            address_to_symbols
                .entry(address)
                .and_modify(|s| {
                    s.insert(identifier.to_owned());
                })
                .or_insert(HashSet::from_iter(iter::once(identifier.to_owned())));
        }

        // Get the text section
        let text_section = elf_file
            .section_header_by_name(".text")
            .context("Failed to parse section table in ELF file")?
            .context("ELF file did not have a .text segment")?;
        let text_section_start = text_section.sh_addr;
        let (code_bytes, compression_header) = elf_file
            .section_data(&text_section)
            .context("Failed to extract code from .text section of ELF file")?;
        ensure!(
            compression_header.is_none(),
            "ELF file had compression header"
        );

        // Get the location of the entry point (relative to start of text section)
        let &entry_point = symbols_to_address
            .get("entry")
            .context(".text section of ELF file did not have entry symbol")?;
        let entry_point_in_text: usize = (entry_point - text_section_start)
            .try_into()
            .context("entry point in text section too large")?;

        // Disassemble the text section into instructions
        let mut decoder = Decoder::with_ip(
            64,
            &code_bytes[entry_point_in_text..],
            entry_point,
            DecoderOptions::NONE,
        );
        let instrs: Vec<_> = decoder.iter().collect();

        let a86_instrs = instrs
            .iter()
            .map(|&x86_instr| x86_instr.try_into())
            .collect::<Vec<Result<Instruction>>>();

        let mut instructions = Vec::with_capacity(a86_instrs.len());

        for (i, x) in a86_instrs.iter().enumerate() {
            match x {
                Ok(t) => instructions.push(*t),
                Err(e) => bail!(
                    "errored at instruction {i} ({:#x?}) with error {e}",
                    instrs[i]
                ),
            };
        }

        let mut memory_map = BiMap::new();
        for (i, instr) in instrs.iter().enumerate() {
            memory_map.insert(instr.ip(), i);
        }

        Ok(Self {
            entry_point,

            instructions,
            memory_map,

            address_to_symbols,
            symbols_to_address,
        })
    }
}
