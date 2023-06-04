use core::{str::from_utf8, cmp::Ordering};
use alloc::vec::Vec;

#[allow(unused_imports)]
use vek::num_traits::real::Real;

pub type Float = f32;
pub type Address = usize;
pub type ColorAddress = [Address; 2];
pub type Couple = vek::vec::repr_c::vec2::Vec2<Float>;
pub const C_ZERO: Couple = Couple::new(0.0, 0.0);

#[derive(Debug, Clone, PartialEq)]
pub struct Argument<T> {
    pub name: Option<T>,
    pub value: Couple,
    pub range: (Couple, Couple),
}

impl<T> Argument<T> {
    pub fn named(name: T, value: Couple) -> Self {
        Self {
            name: Some(name),
            value,
            range: (value, value),
        }
    }

    pub fn unnamed(value: Couple) -> Self {
        Self {
            name: None,
            value,
            range: (value, value),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Output<T> {
    pub name: Option<T>,
    pub address: Address,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Operation {
    Add2,       // add two couples
    Subtract2,  // subtract two couples
    Multiply2,  // multiply two couples
    Divide2,    // divide two couples
    Select2,    // X of op1 and Y of op2
    EachX2,     // X of op1 and X of op2
    EachY2,     // Y of op1 and Y of op2
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

impl Instruction {
    pub fn new(operation: Operation, arg0: Address, arg1: Address, arg2: Address) -> Self {
        Self {
            operation,
            operands: [arg0, arg1, arg2],
        }
    }
}

const OPERATIONS: [Operation; 14] = [
    Operation::Add2,
    Operation::Subtract2,
    Operation::Multiply2,
    Operation::Divide2,
    Operation::Select2,
    Operation::EachX2,
    Operation::EachY2,
    Operation::Polar1,
    Operation::Cartesian1,
    Operation::Cartesian2,
    Operation::Inside3,
    Operation::Swap1,
    Operation::Adjusted3,
    Operation::Clamp3,
];

impl Operation {
    pub fn opcode(self) -> u32 {
        match self {
            Operation::Add2 => 0x0,
            Operation::Subtract2 => 0x1,
            Operation::Multiply2 => 0x2,
            Operation::Divide2 => 0x3,
            Operation::Select2 => 0x4,
            Operation::EachX2 => 0x5,
            Operation::EachY2 => 0x6,
            Operation::Polar1 => 0x7,
            Operation::Cartesian1 => 0x8,
            Operation::Cartesian2 => 0x9,
            Operation::Inside3 => 0xA,
            Operation::Swap1 => 0xB,
            Operation::Adjusted3 => 0xC,
            Operation::Clamp3 => 0xD,
        }
    }

    pub fn number_of_operands(self) -> u32 {
        match self {
            Operation::Add2 => 2,
            Operation::Subtract2 => 2,
            Operation::Multiply2 => 2,
            Operation::Divide2 => 2,
            Operation::Select2 => 2,
            Operation::EachX2 => 2,
            Operation::EachY2 => 2,
            Operation::Polar1 => 1,
            Operation::Cartesian1 => 1,
            Operation::Cartesian2 => 2,
            Operation::Inside3 => 3,
            Operation::Swap1 => 1,
            Operation::Adjusted3 => 3,
            Operation::Clamp3 => 3,
        }
    }

    pub fn as_text(self) -> &'static str {
        match self {
            Operation::Add2 => "Add2",
            Operation::Subtract2 => "Subtract2",
            Operation::Multiply2 => "Multiply2",
            Operation::Divide2 => "Divide2",
            Operation::Select2 => "Select2",
            Operation::EachX2 => "EachX2",
            Operation::EachY2 => "EachY2",
            Operation::Polar1 => "Polar1",
            Operation::Cartesian1 => "Cartesian1",
            Operation::Cartesian2 => "Cartesian2",
            Operation::Inside3 => "Inside3",
            Operation::Swap1 => "Swap1",
            Operation::Adjusted3 => "Adjusted3",
            Operation::Clamp3 => "Clamp3",
        }
    }
}

fn cartesian1(a: Couple) -> (Float, Float) {
    let tmp = a.x.sin_cos();
    (tmp.1 * a.y, (-tmp.0) * a.y)
}

fn add2(a: Couple, b: Couple) -> (Float, Float) {
    (a.x + b.x, a.y + b.y)
}

fn compute(
    instruction: Instruction,
    operands: [Couple; 3],
) -> Couple {
    use Operation::*;

    let [a, b, c] = operands;
    Couple::from(match instruction.operation {
        Add2 => add2(a, b),
        Subtract2 => (a.x - b.x, a.y - b.y),
        Multiply2 => (a.x * b.x, a.y * b.y),
        Divide2 => (a.x / b.x, a.y / b.y),
        Select2 => (a.x, b.y),
        EachX2 => (a.x, b.x),
        EachY2 => (a.y, b.y),
        Polar1 => ((-a.y).atan2(a.x), (a.x * a.x + a.y * a.y).sqrt()),
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
    })
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Stroker {
    pub pattern: Address,
    /// The stroke with is the addition of X and Y at this address
    pub width: Address,
    pub color: ColorAddress,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Triangle {
    pub points: [Address; 3],
    pub colors: [ColorAddress; 3],
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Arc {
    pub start_point: Address,
    pub center: Address,
    pub deltas: Address,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CubicCurve {
    pub points: [Address; 4],
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct QuadraticCurve {
    pub points: [Address; 3],
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Line {
    pub points: [Address; 2],
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PathStep {
    Arc(Arc),
    CubicCurve(CubicCurve),
    QuadraticCurve(QuadraticCurve),
    Line(Line),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RenderingStep<P, B> {
    Clip(P, B),
    Stroke(P, Stroker),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RawRenderingStep {
    Clip(usize, usize),
    Stroke(usize, usize),
}

use ParsingError::*;

#[derive(Debug, Copy, Clone)]
pub enum ParsingError {
    NotARailwayFile,
    TooShort,
    ExcessBytes,
    InvalidStepType,
    InvalidOperation,
    InvalidRenderingStep,
    InvalidName,
    NoArguments,
    InvalidIndex,
}

pub type ParsingResult<T> = Result<T, ParsingError>;

const MAGIC_BYTES: [u8; 4] = [b'R', b'W', b'Y', b'0'];

fn slice<'a>(bytes: &'a [u8], i: &mut usize, len: usize) -> ParsingResult<&'a [u8]> {
    let pos = *i;
    *i += len;
    match bytes.get(pos..*i) {
        Some(bytes) => Ok(bytes),
        None => Err(TooShort),
    }
}

fn read_u32(bytes: &[u8], i: &mut usize) -> ParsingResult<u32> {
    let u8x4 = slice(bytes, i, 4)?;
    let bytes: [u8; 4] = u8x4.try_into().unwrap();
    Ok(u32::from_be_bytes(bytes))
}

fn read_f32(bytes: &[u8], i: &mut usize) -> ParsingResult<f32> {
    let u8x4 = slice(bytes, i, 4)?;
    let bytes: [u8; 4] = u8x4.try_into().unwrap();
    Ok(f32::from_be_bytes(bytes))
}

fn discover_section(bytes: &[u8], i: &mut usize, bytes_per_item: usize) -> ParsingResult<usize> {
    let file_offset = *i;
    *i += (read_u32(bytes, i)? as usize) * bytes_per_item;
    Ok(file_offset)
}

#[derive(Debug, Clone, PartialEq)]
pub struct SerializedProgram<T> {
    storage: T,
    arguments: usize,
    instructions: usize,
    outputs: usize,
    triangles: usize,
    arcs: usize,
    cubic_curves: usize,
    quadratic_curves: usize,
    lines: usize,
    strokers: usize,
    steps: usize,
    paths: usize,
    triangle_indexes: usize,
    backgrounds: usize,
    rendering_steps: usize,
    string_bytes: usize,
}

const QUAD: usize = 4;

/// High Level API
impl<T: AsRef<[u8]>> SerializedProgram<T> {
    pub fn new(storage: T) -> ParsingResult<Self> {
        let bytes = storage.as_ref();
        bytes.strip_prefix(&MAGIC_BYTES).ok_or(NotARailwayFile)?;

        let mut i = MAGIC_BYTES.len();
        let i = &mut i;

        let arguments = discover_section(bytes, i, 7 * QUAD)?;
        let instructions = discover_section(bytes, i, 4 * QUAD)?;
        let outputs = discover_section(bytes, i, 2 * QUAD)?;
        let triangles = discover_section(bytes, i, 9 * QUAD)?;
        let arcs = discover_section(bytes, i, 3 * QUAD)?;
        let cubic_curves = discover_section(bytes, i, 4 * QUAD)?;
        let quadratic_curves = discover_section(bytes, i, 3 * QUAD)?;
        let lines = discover_section(bytes, i, 2 * QUAD)?;
        let strokers = discover_section(bytes, i, 4 * QUAD)?;
        let steps = discover_section(bytes, i, 2 * QUAD)?;
        let paths = discover_section(bytes, i, 2 * QUAD)?;
        let triangle_indexes = discover_section(bytes, i, 1 * QUAD)?;
        let backgrounds = discover_section(bytes, i, 2 * QUAD)?;
        let rendering_steps = discover_section(bytes, i, 3 * QUAD)?;
        let string_bytes = discover_section(bytes, i, 1)?;

        if *i == bytes.len() {
            Ok(Self {
                storage,
                arguments,
                instructions,
                outputs,
                triangles,
                arcs,
                cubic_curves,
                quadratic_curves,
                lines,
                strokers,
                steps,
                paths,
                triangle_indexes,
                backgrounds,
                rendering_steps,
                string_bytes,
            })
        } else {
            Err(ExcessBytes)
        }
    }

    pub fn stack_size(&self) -> usize {
        self.arguments() + self.instructions()
    }

    pub fn compute(&self, stack: &mut [Couple], mut changes: Option<&mut [bool]>) -> ParsingResult<()> {
        let ins_count = self.instructions();
        let mut current = self.arguments();

        for i in 0..ins_count {
            let instruction = self.instruction(i)?;
            let get_op = |a| stack[..current].get(a).ok_or(InvalidOperation);

            let operands = [
                *get_op(instruction.operands[0])?,
                *get_op(instruction.operands[1])?,
                *get_op(instruction.operands[2])?,
            ];

            let result = compute(instruction, operands);

            if result != stack[current] {
                if let Some(changes) = changes.as_mut() {
                    changes[current] = true;
                }
            }

            stack[current] = result;

            current += 1;
        }

        Ok(())
    }

    fn read_usize(&self, i: &mut usize) -> ParsingResult<usize> {
        Ok(read_u32(self.storage.as_ref(), i)? as usize)
    }

    fn read_f32(&self, i: &mut usize) -> ParsingResult<f32> {
        read_f32(self.storage.as_ref(), i)
    }

    fn read_nts<'a>(&'a self, i: &mut usize) -> ParsingResult<Option<&'a str>> {
        let str_offset = self.read_usize(i)?;
        if str_offset != (u32::MAX as usize) {
            let bytes = self.storage.as_ref();
            let str_start = self.string_bytes + QUAD + str_offset;
            let mut len = 0;
            while bytes[str_start + len] != 0 {
                len += 1;
            }
            Ok(Some(from_utf8(&bytes[str_start..][..len]).map_err(|_| InvalidName)?))
        } else {
            Ok(None)
        }
    }

    pub fn arguments(&self) -> usize {
        self.read_usize(&mut self.arguments.clone()).unwrap()
    }

    pub fn argument<'a>(&'a self, i: usize) -> ParsingResult<Argument<&'a str>> {
        self.arguments().checked_sub(i).ok_or(InvalidIndex)?;
        let mut b = self.arguments + QUAD + i * 7 * QUAD;

        let name = self.read_nts(&mut b)?;
        let x     = self.read_f32(&mut b)?;
        let y     = self.read_f32(&mut b)?;
        let min_x = self.read_f32(&mut b)?;
        let max_x = self.read_f32(&mut b)?;
        let min_y = self.read_f32(&mut b)?;
        let max_y = self.read_f32(&mut b)?;
        Ok(Argument {
            name,
            value: Couple::new(x, y),
            range: (Couple::new(min_x, min_y), Couple::new(max_x, max_y)),
        })
    }

    pub fn instructions(&self) -> usize {
        self.read_usize(&mut self.instructions.clone()).unwrap()
    }

    pub fn instruction(&self, i: usize) -> ParsingResult<Instruction> {
        self.instructions().checked_sub(i).ok_or(InvalidIndex)?;
        let mut b = self.instructions + QUAD + i * 4 * QUAD;

        let op = self.read_usize(&mut b)?;
        let a1 = self.read_usize(&mut b)?;
        let a2 = self.read_usize(&mut b)?;
        let a3 = self.read_usize(&mut b)?;
        Ok(Instruction {
            operation: OPERATIONS[op],
            operands: [a1, a2, a3],
        })
    }

    pub fn outputs(&self) -> usize {
        self.read_usize(&mut self.outputs.clone()).unwrap()
    }

    pub fn output<'a>(&'a self, i: usize) -> ParsingResult<Output<&'a str>> {
        self.outputs().checked_sub(i).ok_or(InvalidIndex)?;
        let mut b = self.outputs + QUAD + i * 2 * QUAD;

        let name = self.read_nts(&mut b)?;
        let address = self.read_usize(&mut b)?;
        Ok(Output {
            name,
            address,
        })
    }

    pub fn rendering_steps(&self) -> usize {
        self.read_usize(&mut self.rendering_steps.clone()).unwrap()
    }

    pub fn raw_rendering_step(&self, i: usize) -> ParsingResult<RawRenderingStep> {
        self.rendering_steps().checked_sub(i).ok_or(InvalidIndex)?;
        let mut b = self.rendering_steps + QUAD + i * 3 * QUAD;

        let clip_or_stroke = self.read_usize(&mut b)?;
        let path_index = self.read_usize(&mut b)?;
        let arg_index = self.read_usize(&mut b)?;
        Ok(match clip_or_stroke {
            0 => RawRenderingStep::Clip(path_index, arg_index),
            1 => RawRenderingStep::Stroke(path_index, arg_index),
            _ => unreachable!(),
        })
    }

    pub fn rendering_step<'a>(&'a self, i: usize) -> ParsingResult<RenderingStep<PathIterator<'a, T>, BackgroundIterator<'a, T>>> {
        Ok(match self.raw_rendering_step(i)? {
            RawRenderingStep::Clip(p, i) => RenderingStep::Clip(self.path(p)?, self.background(i)?),
            RawRenderingStep::Stroke(p, i) => RenderingStep::Stroke(self.path(p)?, self.stroker(i)?),
        })
    }
}

/// Low Level API
impl<T: AsRef<[u8]>> SerializedProgram<T> {
    pub fn triangles(&self) -> usize {
        self.read_usize(&mut self.triangles.clone()).unwrap()
    }

    pub fn triangle(&self, i: usize) -> ParsingResult<Triangle> {
        self.triangles().checked_sub(i).ok_or(InvalidIndex)?;
        let mut b = self.triangles + QUAD + i * 9 * QUAD;

        let p0 = self.read_usize(&mut b)?;
        let p1 = self.read_usize(&mut b)?;
        let p2 = self.read_usize(&mut b)?;
        let p0_rg = self.read_usize(&mut b)?;
        let p0_ba = self.read_usize(&mut b)?;
        let p1_rg = self.read_usize(&mut b)?;
        let p1_ba = self.read_usize(&mut b)?;
        let p2_rg = self.read_usize(&mut b)?;
        let p2_ba = self.read_usize(&mut b)?;
        Ok(Triangle {
            points: [p0, p1, p2],
            colors: [[p0_rg, p0_ba], [p1_rg, p1_ba], [p2_rg, p2_ba]],
        })
    }

    pub fn cubic_curves(&self) -> usize {
        self.read_usize(&mut self.cubic_curves.clone()).unwrap()
    }

    pub fn cubic_curve(&self, i: usize) -> ParsingResult<CubicCurve> {
        self.cubic_curves().checked_sub(i).ok_or(InvalidIndex)?;
        let mut b = self.cubic_curves + QUAD + i * 4 * QUAD;

        let p0 = self.read_usize(&mut b)?;
        let p1 = self.read_usize(&mut b)?;
        let p2 = self.read_usize(&mut b)?;
        let p3 = self.read_usize(&mut b)?;
        Ok(CubicCurve {
            points: [p0, p1, p2, p3],
        })
    }

    pub fn arcs(&self) -> usize {
        self.read_usize(&mut self.arcs.clone()).unwrap()
    }

    pub fn arc(&self, i: usize) -> ParsingResult<Arc> {
        self.arcs().checked_sub(i).ok_or(InvalidIndex)?;
        let mut b = self.arcs + QUAD + i * 3 * QUAD;

        let start_point = self.read_usize(&mut b)?;
        let center = self.read_usize(&mut b)?;
        let deltas = self.read_usize(&mut b)?;
        Ok(Arc {
            start_point,
            center,
            deltas,
        })
    }

    pub fn quadratic_curves(&self) -> usize {
        self.read_usize(&mut self.quadratic_curves.clone()).unwrap()
    }

    pub fn quadratic_curve(&self, i: usize) -> ParsingResult<QuadraticCurve> {
        self.quadratic_curves().checked_sub(i).ok_or(InvalidIndex)?;
        let mut b = self.quadratic_curves + QUAD + i * 3 * QUAD;

        let p0 = self.read_usize(&mut b)?;
        let p1 = self.read_usize(&mut b)?;
        let p2 = self.read_usize(&mut b)?;
        Ok(QuadraticCurve {
            points: [p0, p1, p2],
        })
    }

    pub fn lines(&self) -> usize {
        self.read_usize(&mut self.lines.clone()).unwrap()
    }

    pub fn line(&self, i: usize) -> ParsingResult<Line> {
        self.lines().checked_sub(i).ok_or(InvalidIndex)?;
        let mut b = self.lines + QUAD + i * 2 * QUAD;

        let p0 = self.read_usize(&mut b)?;
        let p1 = self.read_usize(&mut b)?;
        Ok(Line {
            points: [p0, p1],
        })
    }

    pub fn strokers(&self) -> usize {
        self.read_usize(&mut self.strokers.clone()).unwrap()
    }

    pub fn stroker(&self, i: usize) -> ParsingResult<Stroker> {
        self.strokers().checked_sub(i).ok_or(InvalidIndex)?;
        let mut b = self.strokers + QUAD + i * 4 * QUAD;

        let pattern = self.read_usize(&mut b)?;
        let width = self.read_usize(&mut b)?;
        let rg = self.read_usize(&mut b)?;
        let ba = self.read_usize(&mut b)?;
        Ok(Stroker {
            pattern,
            width,
            color: [rg, ba],
        })
    }

    pub fn paths(&self) -> usize {
        self.read_usize(&mut self.paths.clone()).unwrap()
    }

    pub fn raw_path(&self, i: usize) -> ParsingResult<RawPath> {
        self.paths().checked_sub(i).ok_or(InvalidIndex)?;
        let mut b = self.paths + QUAD + i * 2 * QUAD;

        let step_offset = self.steps + QUAD + self.read_usize(&mut b)? * 2 * QUAD;
        let stop_before = step_offset + (self.read_usize(&mut b)? * 2 * QUAD);
        Ok(RawPath {
            step_offset,
            stop_before,
        })
    }

    pub fn path<'a>(&'a self, i: usize) -> ParsingResult<PathIterator<'a, T>> {
        let RawPath { step_offset, stop_before } = self.raw_path(i)?;
        Ok(PathIterator {
            program: self,
            step_offset,
            stop_before,
        })
    }

    pub fn triangle_indexes(&self) -> usize {
        self.read_usize(&mut self.triangle_indexes.clone()).unwrap()
    }

    pub fn triangle_index(&self, i: usize) -> ParsingResult<usize> {
        self.triangle_indexes().checked_sub(i).ok_or(InvalidIndex)?;
        self.read_usize(&mut (self.triangle_indexes + QUAD + i * 1 * QUAD).clone())
    }

    pub fn backgrounds(&self) -> usize {
        self.read_usize(&mut self.backgrounds.clone()).unwrap()
    }

    pub fn raw_background(&self, i: usize) -> ParsingResult<RawBackground> {
        self.backgrounds().checked_sub(i).ok_or(InvalidIndex)?;
        let mut b = self.backgrounds + QUAD + i * 2 * QUAD;

        let triangle_index_offset = self.read_usize(&mut b)?;
        let stop_before = triangle_index_offset + self.read_usize(&mut b)?;
        Ok(RawBackground {
            triangle_index_offset,
            stop_before,
        })
    }

    pub fn background<'a>(&'a self, i: usize) -> ParsingResult<BackgroundIterator<'a, T>> {
        let RawBackground { triangle_index_offset, stop_before } = self.raw_background(i)?;
        Ok(BackgroundIterator {
            program: self,
            triangle_index_offset,
            stop_before,
        })
    }
}

pub struct RawPath {
    pub step_offset: usize,
    pub stop_before: usize,
}

pub struct RawBackground {
    pub triangle_index_offset: usize,
    pub stop_before: usize,
}

pub struct PathIterator<'a, T> {
    program: &'a SerializedProgram<T>,
    step_offset: usize,
    stop_before: usize,
}

impl<'a, T: AsRef<[u8]>> Iterator for PathIterator<'a, T> {
    type Item = ParsingResult<PathStep>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.step_offset < self.stop_before {
            let step_type = self.program.read_usize(&mut self.step_offset);
            let index = self.program.read_usize(&mut self.step_offset);
            if let (Ok(step_type), Ok(index)) = (step_type, index) {
                let result = match step_type {
                    0 => self.program.arc(index).map(|_| ()),
                    1 => self.program.cubic_curve(index).map(|_| ()),
                    2 => self.program.quadratic_curve(index).map(|_| ()),
                    3 => self.program.line(index).map(|_| ()),
                    _ => Err(InvalidStepType),
                };

                if let Err(e) = result {
                    Some(Err(e))
                } else {
                    Some(match step_type {
                        0 => Ok(PathStep::Arc(self.program.arc(index).unwrap())),
                        1 => Ok(PathStep::CubicCurve(self.program.cubic_curve(index).unwrap())),
                        2 => Ok(PathStep::QuadraticCurve(self.program.quadratic_curve(index).unwrap())),
                        3 => Ok(PathStep::Line(self.program.line(index).unwrap())),
                        _ => unreachable!(),
                    })
                }
            } else {
                match (step_type, index) {
                    (Err(e), _) => Some(Err(e)),
                    (Ok(_), Err(e)) => Some(Err(e)),
                    _ => unreachable!(),
                }
            }
        } else {
            None
        }
    }
}

pub struct BackgroundIterator<'a, T> {
    program: &'a SerializedProgram<T>,
    triangle_index_offset: usize,
    stop_before: usize,
}

impl<'a, T: AsRef<[u8]>> Iterator for BackgroundIterator<'a, T> {
    type Item = ParsingResult<Triangle>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.triangle_index_offset < self.stop_before {
            match self.program.triangle_index(self.triangle_index_offset) {
                Ok(triangle_index) => {
                    self.triangle_index_offset += 1;
                    Some(self.program.triangle(triangle_index))
                },
                Err(e) => Some(Err(e))
            }
        } else {
            None
        }
    }
}

fn bytes(n: usize) -> [u8; 4] {
    (n as u32).to_be_bytes()
}

pub fn serialize<S: AsRef<str>, P: AsRef<[PathStep]>, B: AsRef<[Triangle]>>(
    arguments: &[Argument<S>],
    instructions: &[Instruction],
    outputs: &[Output<S>],
    rendering_steps: &[RenderingStep<P, B>],
) -> Vec<u8> {
    let mut output = Vec::new();

    let mut write_fn = |slice: [u8; 4]| output.extend_from_slice(&slice);

    write_fn(MAGIC_BYTES);

    let mut string_section = Vec::new();

    write_fn(bytes(arguments.len()));
    for i in arguments {
        let offset = if let Some(s) = &i.name {
            let offset = string_section.len();
            string_section.extend_from_slice(s.as_ref().as_bytes());
            string_section.push(0);
            offset
        } else {
            u32::MAX as usize
        };
        write_fn(bytes(offset));
        write_fn(i.value.x.to_be_bytes());
        write_fn(i.value.y.to_be_bytes());
        write_fn(i.range.0.x.to_be_bytes());
        write_fn(i.range.1.x.to_be_bytes());
        write_fn(i.range.0.y.to_be_bytes());
        write_fn(i.range.1.y.to_be_bytes());
    }

    write_fn(bytes(instructions.len()));
    for i in instructions {
        write_fn(i.operation.opcode().to_be_bytes());
        write_fn(bytes(i.operands[0]));
        write_fn(bytes(i.operands[1]));
        write_fn(bytes(i.operands[2]));
    }

    write_fn(bytes(outputs.len()));
    for i in outputs {
        let offset = if let Some(s) = &i.name {
            let offset = string_section.len();
            string_section.extend_from_slice(s.as_ref().as_bytes());
            string_section.push(0);
            offset
        } else {
            u32::MAX as usize
        };
        write_fn(bytes(offset));
        write_fn(bytes(i.address));
    }

    let mut triangles = Vec::new();
    let mut triangle_indexes = Vec::new();
    let mut backgrounds = Vec::new();
    let mut paths = Vec::new();
    let mut arcs = Vec::new();
    let mut cubic_curves = Vec::new();
    let mut quadratic_curves = Vec::new();
    let mut lines = Vec::new();
    let mut strokers = Vec::new();
    let mut flat_rendering_steps = Vec::new();
    let mut steps = Vec::new();

    fn find_or_push<T: 'static + Eq>(vec: &mut Vec<T>, obj: T) -> usize {
        vec.iter().position(|o| o == &obj).unwrap_or_else(|| {
            let index = vec.len();
            vec.push(obj);
            index
        })
    }

    fn find_or_push_slice<T: 'static + Eq + Clone>(vec: &mut Vec<T>, slice: &[T]) -> [usize; 2] {
        [vec.windows(slice.len()).position(|s| s == slice).unwrap_or_else(|| {
            let index = vec.len();
            vec.extend_from_slice(slice);
            index
        }), slice.len()]
    }

    for step in rendering_steps {
        let (clip_or_stroke, path, arg_index) = if let RenderingStep::Clip(path, background) = step {

            let mut indexes = Vec::with_capacity(background.as_ref().len());
            for triangle in background.as_ref() {
                let triangle_index = find_or_push(&mut triangles, [
                    triangle.points[0],
                    triangle.points[1],
                    triangle.points[2],
                    triangle.colors[0][0],
                    triangle.colors[0][1],
                    triangle.colors[1][0],
                    triangle.colors[1][1],
                    triangle.colors[2][0],
                    triangle.colors[2][1],
                ]);
                indexes.push([triangle_index]);
            }

            (0, path, find_or_push(&mut backgrounds, find_or_push_slice(&mut triangle_indexes, &indexes)))

        } else if let RenderingStep::Stroke(path, s) = step {

            (1, path, find_or_push(&mut strokers, [s.pattern, s.width, s.color[0], s.color[1]]))

        } else {
            unreachable!()
        };

        let mut tmp_steps = Vec::with_capacity(path.as_ref().len());
        for step in path.as_ref() {
            tmp_steps.push(match step {
                PathStep::Arc(arc) => [0, find_or_push(&mut arcs, [arc.start_point, arc.center, arc.deltas])],
                PathStep::CubicCurve(curve) => [1, find_or_push(&mut cubic_curves, curve.points)],
                PathStep::QuadraticCurve(curve) => [2, find_or_push(&mut quadratic_curves, curve.points)],
                PathStep::Line(line) => [3, find_or_push(&mut lines, line.points)],
            });
        }
        let path_index = find_or_push(&mut paths, find_or_push_slice(&mut steps, &tmp_steps));

        flat_rendering_steps.push([clip_or_stroke, path_index, arg_index]);
    }

    fn for_each<const N: usize, F: FnMut([u8; 4])>(write_fn: &mut F, array: &[[usize; N]]) {
        write_fn(bytes(array.len()));
        for addresses in array {
            for address in addresses {
                write_fn(bytes(*address));
            }
        }
    }

    for_each(&mut write_fn, &triangles);
    for_each(&mut write_fn, &arcs);
    for_each(&mut write_fn, &cubic_curves);
    for_each(&mut write_fn, &quadratic_curves);
    for_each(&mut write_fn, &lines);
    for_each(&mut write_fn, &strokers);
    for_each(&mut write_fn, &steps);
    for_each(&mut write_fn, &paths);
    for_each(&mut write_fn, &triangle_indexes);
    for_each(&mut write_fn, &backgrounds);
    for_each(&mut write_fn, &flat_rendering_steps);

    write_fn(bytes(string_section.len()));
    output.extend_from_slice(&string_section);

    output
}
