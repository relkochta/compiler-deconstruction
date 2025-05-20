use std::collections::VecDeque;

use anyhow::Result;
use anyhow::bail;

use crate::{
    a86::{Address, Arg, Instruction, Program as A86Program, Register},
    loot::{Datum, Defn, Expr, Program as LootProgram, Operation},
};

pub fn parse_const(lit: u64) -> Option<Expr> {
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
        0b011000 => Some(Expr::Literal(Datum::Boolean(true))),
        0b111000 => Some(Expr::Literal(Datum::Boolean(false))),
        0b1111000 => Some(Expr::Op(Operation::Void)),
        lit if lit & 0b11111 == 0b01000 => Some(Expr::Literal(Datum::Character(char::from_u32(
            (lit >> 5).try_into().unwrap(),
        )?))),
        lit if lit & 0b1111 == 0 => Some(Expr::Literal(Datum::Integer((lit >> 4) as i64))),
        _ => None,
    }
}

pub fn parse_expr(
    program: &A86Program,
    position: usize,
    stop: Option<usize>,
    stack: &mut Vec<Expr>,
) -> Result<(Expr, usize)> {
    let mut expr_list = Vec::new();
    let mut pos = position;
    let err_label: Address = program.symbol_to_address("err").unwrap();

    while match stop {
        Some(stop) => pos < stop,
        None => true,
    } {
        // peek on the next instructions
        let (expr, new_pos) = match program.instructions()[pos..] {
            [
                Instruction::Mov(Arg::Register(Register::Eax | Register::Rax), Arg::Literal(lit)),
                ..,
            ] => (parse_const(lit).unwrap(), pos + 1),
            [
                // pad-stack + call + unpad-stack
                Instruction::Mov(Arg::Register(Register::R15), Arg::Register(Register::Rsp)),
                Instruction::And(Arg::Register(Register::R15), Arg::Literal(0x8)),
                Instruction::Sub(Arg::Register(Register::Rsp), Arg::Register(Register::R15)),
                Instruction::Call(addr),
                Instruction::Add(Arg::Register(Register::Rsp), Arg::Register(Register::R15)),
                ..,
            ] => {
                if addr == program.symbol_to_address("read_byte").unwrap() {
                    (Expr::Op(Operation::ReadByte), pos + 5)
                } else if addr == program.symbol_to_address("peek_byte").unwrap() {
                    (Expr::Op(Operation::PeekByte), pos + 5)
                } else {
                    unimplemented!()
                }
            }
            [
                // type check for int
                Instruction::Mov(Arg::Register(Register::R9), Arg::Register(Register::Rax)),
                Instruction::And(Arg::Register(Register::R9), Arg::Literal(0xf)),
                Instruction::Cmp(Arg::Register(Register::R9), Arg::Literal(0x0)),
                Instruction::Jne(Arg::Address(lab)),
                ..,
            ] => {
                if lab != err_label {
                    bail!("expected jump to err label")
                }
                match program.instructions()[pos + 4..] {
                    [
                        Instruction::Add(Arg::Register(Register::Rax), Arg::Literal(0x10)),
                        ..,
                    ] => {
                        // looks like an Add1
                        let v = expr_list.pop();
                        (
                            Expr::Op(Operation::Add1(
                                Box::new(v.unwrap()),
                            )),
                            pos + 5,
                        )
                    }
                    _ => unimplemented!(),
                }
            }
            [
                Instruction::Push(Arg::Register(Register::Eax | Register::Rax)),
                ..,
            ] => {
                // current expression got pushed, start parsing a new one
                let mut expr = expr_list.pop().unwrap();
                while expr_list.len() > 0 {
                    expr = Expr::Begin(Box::new(expr_list.pop().unwrap()), Box::new(expr));
                }
                stack.push(expr);

                parse_expr(program, pos + 1, None, stack)?
            }
            [
                // pop + type check r8 and rax for int
                Instruction::Pop(Arg::Register(Register::R8)),
                Instruction::Mov(Arg::Register(Register::R9), Arg::Register(Register::R8)),
                Instruction::And(Arg::Register(Register::R9), Arg::Literal(0xf)),
                Instruction::Cmp(Arg::Register(Register::R9), Arg::Literal(0x0)),
                Instruction::Jne(Arg::Address(lab1)),
                Instruction::Mov(Arg::Register(Register::R9), Arg::Register(Register::Rax)),
                Instruction::And(Arg::Register(Register::R9), Arg::Literal(0xf)),
                Instruction::Cmp(Arg::Register(Register::R9), Arg::Literal(0x0)),
                Instruction::Jne(Arg::Address(lab2)),
                ..,
            ] => {
                if lab1 != err_label || lab2 != err_label {
                    bail!("expected jump to err label")
                }
                match program.instructions()[pos + 9..] {
                    [
                        Instruction::Add(Arg::Register(Register::Rax), Arg::Register(Register::R8)),
                        ..,
                    ] => {
                        // looks like a Plus
                        let arg1 = stack.pop();
                        let arg2 = expr_list.pop();
                        (
                            Expr::Op(Operation::Plus(
                                Box::new(arg1.unwrap()),
                                Box::new(arg2.unwrap()),
                            )),
                            pos + 10,
                        )
                    }
                    _ => unimplemented!(),
                }
            }
            [
                Instruction::Cmp(Arg::Register(Register::Eax | Register::Rax), Arg::Literal(_)),
                Instruction::Je(Arg::Address(if_false)),
                ..,
            ] => {
                let jmp_loc = program.address_to_index(if_false).unwrap() - 1;
                // We are in an if statement.
                let expr_if_true = parse_expr(program, pos + 2, Some(jmp_loc), stack)?.0;

                let if_end = match program.instructions()[jmp_loc] {
                    Instruction::Jmp(Arg::Address(i)) => program.address_to_index(i).unwrap(),
                    _ => bail!("parsing failed while trying to find end jump for if statement"),
                };

                let expr_if_false = parse_expr(
                    program,
                    program.address_to_index(if_false).unwrap(),
                    Some(if_end),
                    stack
                )?
                .0;

                let v = expr_list.pop();
                (
                    Expr::If(
                        Box::new(v.unwrap()),
                        Box::new(expr_if_true),
                        Box::new(expr_if_false),
                    ),
                    if_end,
                )
            }
            _ => (Expr::Unknown, pos),
        };

        pos = new_pos;
        match expr {
            Expr::Unknown => {
                break;
            }
            _ => {
                expr_list.push(expr);
            }
        }
    }

    let mut expr = expr_list.pop().unwrap();
    while expr_list.len() > 0 {
        expr = Expr::Begin(Box::new(expr_list.pop().unwrap()), Box::new(expr));
    }

    Ok((expr, pos))
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
    let mut stack = Vec::new();
    Ok(LootProgram {
        defines,
        expr: Box::new(parse_expr(program, expr_start, Some(end), &mut stack)?.0),
    })
}
