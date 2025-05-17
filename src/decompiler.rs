use std::collections::VecDeque;

use anyhow::Result;
use anyhow::bail;

use crate::{
    a86::{Arg, Instruction, Program as A86Program, Register},
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

pub fn parse_full(program: &A86Program, position: usize, stop: usize) -> Result<Expr> {
    let (initial_expr, new_pos) = parse_expr(program, position)?;

    let mut expr_set = VecDeque::new();
    expr_set.push_back(initial_expr);
    let mut pos = new_pos;

    while pos < stop {
        // peek on the next expression
        match program.instructions()[pos..] {
            [
                Instruction::Cmp(Arg::Register(Register::Eax | Register::Rax), Arg::Literal(_)),
                Instruction::Je(Arg::Address(if_false)),
                ..,
            ] => {
                let jmp_loc = program.address_to_index(if_false).unwrap() - 1;
                // We are in an if statement.
                let expr_if_true = parse_full(program, pos + 2, jmp_loc)?;

                let if_end = match program.instructions()[jmp_loc] {
                    Instruction::Jmp(Arg::Address(i)) => program.address_to_index(i).unwrap(),
                    _ => bail!("parsing failed while trying to find end jump for if statement"),
                };

                let expr_if_false =
                    parse_full(program, program.address_to_index(if_false).unwrap(), if_end)?;

                let v = expr_set.pop_back();
                expr_set.push_back(Expr::If(
                    Box::new(v.unwrap()),
                    Box::new(expr_if_true),
                    Box::new(expr_if_false),
                ));
                pos = if_end;
            }
            [..] => bail!("what the helly"),
        }
    }

    (Ok(expr_set.pop_back().unwrap()))
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

    //     let (expr2, new_pos) = match program.instructions()[new_pos..] {
    //         [
    //             Instruction::Cmp(Arg::Register(Register::Eax | Register::Rax), Arg::Literal(_)),
    //             Instruction::Je(Arg::Address(if_false)),
    //             ..,
    //         ] => {
    //             let (expr_if_true, new_pos) = parse_expr(
    //                 program,
    //                 new_pos + 2,
    //                 Some(program.address_to_index(if_false).unwrap() - 1),
    //             )?;
    //
    //             let if_end = match program.instructions()[new_pos] {
    //                 Instruction::Jmp(Arg::Address(i)) => program.address_to_index(i).unwrap(),
    //                 _ => bail!("parsing failed while trying to find end jump for if statement"),
    //             };
    //
    //             let (expr_if_false, new_pos2) = parse_expr(
    //                 program,
    //                 program.address_to_index(if_false).unwrap(),
    //                 Some(if_end),
    //             )?;
    //             (
    //                 Expr::If(
    //                     Box::new(initial_expr),
    //                     Box::new(expr_if_true),
    //                     Box::new(expr_if_false),
    //                 ),
    //                 new_pos2,
    //             )
    //         }
    //         _ => (initial_expr, new_pos),
    //     };
    //
    //     if let Some(x) = stop {
    //         if new_pos > x {
    //             bail!(
    //                 "expression parsed goes to {:x}, past end of expression {:x}",
    //                 program.index_to_address(new_pos).unwrap(),
    //                 program.index_to_address(x).unwrap()
    //             );
    //         } else if new_pos < x {
    //             // TODO: begin expression check issue here lol
    //             bail!("expression parsed is smaller than if branch");
    //         } else {
    //             Ok((expr2, new_pos))
    //         }
    //     } else {
    //         Ok((expr2, new_pos))
    //     }
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
        expr: Box::new(parse_full(program, expr_start, end)?),
    })
}
