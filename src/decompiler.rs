use std::collections::VecDeque;

use anyhow::Result;
use anyhow::bail;

use crate::{
    a86::{Arg, Instruction, Program as A86Program, Register, Address},
    loot::{Datum, Defn, Expr, Program as LootProgram},
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

pub fn parse_range(program: &A86Program, position: usize, stop: usize) -> Result<Expr> {
    let (initial_expr, new_pos) = parse_expr(program, position)?;

    let mut expr_set = VecDeque::new();
    expr_set.push_back(initial_expr);
    let mut pos = new_pos;

    let err_label: Address = program.symbol_to_address("err").unwrap();

    while pos < stop {
        // peek on the next expression
        match program.instructions()[pos..] {
            [
                Instruction::Mov(Arg::Register(Register::R9), Arg::Register(Register::Rax)),
                Instruction::And(Arg::Register(Register::R9), Arg::Literal(0xf)),
                Instruction::Cmp(Arg::Register(Register::R9), Arg::Literal(0x0)),
                Instruction::Jne(Arg::Address(lab)),
                ..,
            ] => {
                // We are in an Op1
                if lab != err_label {
                    bail!("wtf")
                }
                todo!()
            }
            /*
            [
                Instruction::Push(Arg::Register(Register::Eax | Register::Rax)),
                ..,
            ] => {
                // We are in an Op2 or Op3
            }
            */
            [
                Instruction::Cmp(Arg::Register(Register::Eax | Register::Rax), Arg::Literal(_)),
                Instruction::Je(Arg::Address(if_false)),
                ..,
            ] => {
                let jmp_loc = program.address_to_index(if_false).unwrap() - 1;
                // We are in an if statement.
                let expr_if_true = parse_range(program, pos + 2, jmp_loc)?;

                let if_end = match program.instructions()[jmp_loc] {
                    Instruction::Jmp(Arg::Address(i)) => program.address_to_index(i).unwrap(),
                    _ => bail!("parsing failed while trying to find end jump for if statement"),
                };

                let expr_if_false =
                    parse_range(program, program.address_to_index(if_false).unwrap(), if_end)?;

                let v = expr_set.pop_back();
                expr_set.push_back(Expr::If(
                    Box::new(v.unwrap()),
                    Box::new(expr_if_true),
                    Box::new(expr_if_false),
                ));
                pos = if_end;
            }
            [..] => {
                // We are in a begin statement (hopefully).
                let v = expr_set.pop_back();
                let next_expr = parse_range(program, pos, stop);
                expr_set.push_back(Expr::Begin(
                    Box::new(v.unwrap()),
                    Box::new(next_expr.unwrap()),
                ));
                pos = stop;
            }
        }
    }

    Ok(expr_set.pop_back().unwrap())
}

pub fn parse_expr(program: &A86Program, position: usize) -> Result<(Expr, usize)> {
    let (initial_expr, new_pos) = match program.instructions()[position..] {
        [
            Instruction::Mov(Arg::Register(Register::Eax | Register::Rax), Arg::Literal(lit)),
            ..,
        ] => (Expr::Literal(parse_const(lit).unwrap()), position + 1),
        _ => (Expr::Unknown, position),
    };

    Ok((initial_expr, new_pos))
}

pub fn parse_defines(program: &A86Program, position: usize) -> (Vec<Defn>, usize) {
    // TODO: implement this

    (Vec::new(), position + 1) // skip add rbx
}

pub fn parse(program: &A86Program) -> Result<LootProgram> {
    let (defines, expr_start) = match program.instructions()[0..3] {
        [
            Instruction::Push(Arg::Register(Register::Rbx)),
            Instruction::Push(Arg::Register(Register::R15)),
            Instruction::Mov(Arg::Register(Register::Rbx), Arg::Register(Register::Rdi)),
        ] => parse_defines(program, 3),
        _ => bail!("Unable to parse loot program"),
    };
    let end = program
        .address_to_index(program.symbol_to_address("err").unwrap())
        .unwrap()
        - 4;
    Ok(LootProgram {
        defines,
        expr: Box::new(parse_range(program, expr_start, end)?),
    })
}
