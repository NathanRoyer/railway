use crate::Address;
use crate::Couple;
use crate::Float;
use crate::Program;
use crate::C_ZERO;

use core::cmp::Ordering;
use alloc::vec::Vec;
use alloc::string::String;

use Operation::*;

use num_traits::real::Real;

#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub name: Option<String>,
    pub value: Couple,
    pub range: (Couple, Couple),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Output {
    pub name: Option<String>,
    pub address: Address,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Operation {
    Add2,       // add two couples
    Subtract2,  // subtract two couples
    Multiply2,  // multiply two couples
    Divide2,    // divide two couples
    Select2,    // X of op1 and Y of op2
    Polar1,     // convert cartesian coordinates to polar ones
    Cartesian1, // convert polar coordinates to cartesian ones
    Cartesian2, // same but then add to a couple
    Inside3,    // (1, 0) if op1 is inside of rectangle (op2 → op3), else (0, 1)
    Swap1,      // swap X and Y
    Adjusted3,  // = a * c.x + b * c.y
    Clamp3,     // op1 clamped (op2 = min; op3 = max)
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Instruction {
    pub operation: Operation,
    pub operands: [Address; 3],
}

pub const OPERATIONS: [Operation; 12] = [
    Add2, Subtract2, Multiply2, Divide2, Select2, Polar1, Cartesian1, Cartesian2, Inside3, Swap1,
    Adjusted3, Clamp3,
];

impl Operation {
    pub fn opcode(self) -> u32 {
        match self {
            Add2 => 0x0,
            Subtract2 => 0x1,
            Multiply2 => 0x2,
            Divide2 => 0x3,
            Select2 => 0x4,
            Polar1 => 0x5,
            Cartesian1 => 0x6,
            Cartesian2 => 0x7,
            Inside3 => 0x8,
            Swap1 => 0x9,
            Adjusted3 => 0xA,
            Clamp3 => 0xB,
        }
    }
}

pub const OPERANDS: [usize; 12] = [2, 2, 2, 2, 2, 1, 1, 2, 3, 1, 3, 3];

fn cartesian1(a: Couple) -> (Float, Float) {
    let tmp = a.x.sin_cos();
    (tmp.1 * a.y, tmp.0 * a.y)
}

fn add2(a: Couple, b: Couple) -> (Float, Float) {
    (a.x + b.x, a.y + b.y)
}

pub fn compute(program: &Program, stack: &mut Vec<Couple>) {
    stack.resize(program.arguments.len(), C_ZERO);
    for instruction in &program.instructions {
        let a = stack[instruction.operands[0] as usize];
        let b = stack[instruction.operands[1] as usize];
        let c = stack[instruction.operands[2] as usize];
        stack.push(Couple::from(match instruction.operation {
            Add2 => add2(a, b),
            Subtract2 => (a.x - b.x, a.y - b.y),
            Multiply2 => (a.x * b.x, a.y * b.y),
            Divide2 => (a.x / b.x, a.y / b.y),
            Select2 => (a.x, b.y),
            Polar1 => (a.y.atan2(a.x), (a.x * a.x + a.y * a.y).sqrt()),
            Cartesian1 => cartesian1(a),
            Cartesian2 => add2(Couple::from(cartesian1(a)), b),
            Inside3 => {
                let bx = a.x.partial_cmp(&b.x);
                let by = a.y.partial_cmp(&b.y);
                let cx = a.x.partial_cmp(&c.x);
                let cy = a.y.partial_cmp(&c.y);
                match (bx, by, cx, cy) {
                    (
                        Some(Ordering::Greater),
                        Some(Ordering::Greater),
                        Some(Ordering::Less),
                        Some(Ordering::Less),
                    ) => (1f32, 0f32),
                    _ => (0f32, 1f32),
                }
            }
            Swap1 => (a.y, a.x),
            Adjusted3 => (a.x * c.x + b.x * c.y, a.y * c.x + b.y * c.y),
            Clamp3 => (a.x.clamp(b.x, c.x), a.y.clamp(b.y, c.y)),
        }));
    }
}
