use anyhow::anyhow;

use crate::{a86::{self, Instruction, Program}, loot::{self, Datum, Expr}};

pub fn parse_datum(program: &a86::Program, position: usize) -> Datum {

    todo!()
}

pub fn parse_expr(program: &a86::Program, position: usize) -> Expr {

    todo!()
}

pub fn parse(program: &a86::Program) -> Result<loot::Program> {
    match program.instructions() {

        _ => anyhow!("Unable to parse loot program")
    }
}
