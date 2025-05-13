use anyhow::Result;
use anyhow::bail;

use crate::{
    a86::{self, Arg, Instruction, Program, Register},
    loot::{self, Datum, Defn, Expr, Program as LootProgram},
};

pub fn parse_const(lit: u64) -> Option<Datum> {
    /*
      Bit layout of values

      Values are either:
      - Immediates: end in #b000
      - Pointers

      Immediates are either
      - Integers:   end in  #b0 000
      - Characters: end in #b01 000
      - True:              #b11 000
      - False:           #b1 11 000
      - Eof:            #b10 11 000
      - Void:           #b11 11 000
      - Empty:         #b100 11 000
    */
    match lit {
        0b011000 => Some(Datum::Boolean(true)),
        0b111000 => Some(Datum::Boolean(false)),
        lit if lit & 0b11111 == 0b01000 => Some(Datum::Character(char::from_u32(
            (lit >> 5).try_into().unwrap(),
        )?)),
        lit if lit & 0b1111 == 0 => Some(Datum::Integer((lit >> 4) as i64)),
        _ => None,
    }
}

pub fn parse_expr(program: &a86::Program, position: usize) -> Expr {
    match program.instructions()[position..] {
        [
            Instruction::Mov(Arg::Register(Register::Rax), Arg::Literal(lit)),
            ..,
        ] => Expr::Literal(parse_const(lit).unwrap()),
        [
            Instruction::Mov(Arg::Register(Register::Eax), Arg::Literal(lit)),
            ..,
        ] => Expr::Literal(parse_const(lit).unwrap()),
        _ => Expr::Unknown,
    }
}

pub fn parse_defines(program: &a86::Program, position: usize) -> (Vec<Defn>, usize) {
    // TODO: implement this

    (Vec::new(), position + 1) // skip add rbx
}

pub fn parse(program: &a86::Program) -> Result<loot::Program> {
    let (defines, expr_start) = match program.instructions()[0..3] {
        [
            Instruction::Push(Arg::Register(Register::Rbx)),
            Instruction::Push(Arg::Register(Register::R15)),
            Instruction::Mov(Arg::Register(Register::Rbx), Arg::Register(Register::Rdi)),
        ] => parse_defines(program, 3),
        _ => bail!("Unable to parse loot program"),
    };
    Ok(LootProgram {
        defines: defines,
        expr: Box::new(parse_expr(program, expr_start)),
    })
}
