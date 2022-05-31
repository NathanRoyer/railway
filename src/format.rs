use crate::Address;
use crate::Arc;
use crate::Argument;
use crate::Couple;
use crate::CubicCurve;
use crate::Instruction;
use crate::Line;
use crate::Output;
use crate::Path;
use crate::Program;
use crate::QuadraticCurve;
use crate::RenderingStep;
use crate::Stroker;
use crate::Triangle;
use crate::OPERATIONS;
use crate::STEP_TYPES;

use std::io::Result as IoResult;
use std::io::Write;
use std::mem::size_of;

use ParsingError::*;

#[derive(Debug, Copy, Clone)]
pub enum ParsingError {
    NotARailwayFile,
    TooShort,
    InvalidStepType,
    InvalidOperation,
    InvalidRenderingStep,
    InvalidName,
    NoArguments,
    InvalidIndex,
}

const MAGIC_BYTES: [u8; 4] = [b'R', b'W', b'Y', b'0'];

/// this function will not check for invalid indexes;
/// but Program::parse() will.
pub fn parse(bytes: &[u8]) -> Result<Program, ParsingError> {
    let bytes = match bytes.strip_prefix(&MAGIC_BYTES) {
        Some(bytes) => Ok(bytes),
        None => Err(NotARailwayFile),
    }?;
    let mut i = 0;
    let i = &mut i;

    let mut arguments = Vec::with_capacity(try_u32_usize(bytes, i)?);
    let mut arg_n_len = Vec::with_capacity(arguments.capacity());
    for _ in 0..arguments.capacity() {
        arg_n_len.push(try_u32_usize(bytes, i)?);
        let x = try_f32(bytes, i)?;
        let y = try_f32(bytes, i)?;
        let min_x = try_f32(bytes, i)?;
        let max_x = try_f32(bytes, i)?;
        let min_y = try_f32(bytes, i)?;
        let max_y = try_f32(bytes, i)?;
        arguments.push(Argument {
            name: None,
            value: Couple::new(x, y),
            range: (Couple::new(min_x, min_y), Couple::new(max_x, max_y)),
        });
    }

    let mut instructions = Vec::with_capacity(try_u32_usize(bytes, i)?);
    for _ in 0..instructions.capacity() {
        let opcode = try_u32_usize(bytes, i)?;
        let operation = *OPERATIONS.get(opcode).ok_or(InvalidOperation)?;
        let a = try_u32_addr(bytes, i)?;
        let b = try_u32_addr(bytes, i)?;
        let c = try_u32_addr(bytes, i)?;
        let operands = [a, b, c];
        instructions.push(Instruction {
            operation,
            operands,
        });
    }

    let mut outputs = Vec::with_capacity(try_u32_usize(bytes, i)?);
    let mut output_n_len = Vec::with_capacity(outputs.capacity());
    for _ in 0..outputs.capacity() {
        output_n_len.push(try_u32_usize(bytes, i)?);
        let address = try_u32_addr(bytes, i)?;
        outputs.push(Output {
            name: None,
            address,
        });
    }

    let mut triangles = Vec::with_capacity(try_u32_usize(bytes, i)?);
    for _ in 0..triangles.capacity() {
        let addresses = try_n_u32_addr(bytes, i, 9)?;
        let points = [addresses[0], addresses[1], addresses[2]];
        let p1c = [addresses[3], addresses[4]];
        let p2c = [addresses[5], addresses[6]];
        let p3c = [addresses[7], addresses[8]];
        triangles.push(Triangle {
            points,
            colors: [p1c, p2c, p3c],
        });
    }

    let mut arcs = Vec::with_capacity(try_u32_usize(bytes, i)?);
    for _ in 0..arcs.capacity() {
        let addresses = try_n_u32_addr(bytes, i, 3)?;
        arcs.push(Arc {
            center: addresses[0],
            angular_range: addresses[1],
            radii: addresses[2],
        });
    }

    let mut cubic_curves = Vec::with_capacity(try_u32_usize(bytes, i)?);
    for _ in 0..cubic_curves.capacity() {
        let addresses = try_n_u32_addr(bytes, i, 4)?;
        cubic_curves.push(CubicCurve {
            points: [addresses[0], addresses[1], addresses[2], addresses[3]],
        });
    }

    let mut quadratic_curves = Vec::with_capacity(try_u32_usize(bytes, i)?);
    for _ in 0..quadratic_curves.capacity() {
        let addresses = try_n_u32_addr(bytes, i, 3)?;
        quadratic_curves.push(QuadraticCurve {
            points: [addresses[0], addresses[1], addresses[2]],
        });
    }

    let mut lines = Vec::with_capacity(try_u32_usize(bytes, i)?);
    for _ in 0..lines.capacity() {
        let addresses = try_n_u32_addr(bytes, i, 2)?;
        lines.push(Line {
            points: [addresses[0], addresses[1]],
        });
    }

    let mut strokers = Vec::with_capacity(try_u32_usize(bytes, i)?);
    for _ in 0..strokers.capacity() {
        let addresses = try_n_u32_addr(bytes, i, 4)?;
        strokers.push(Stroker {
            pattern: addresses[0],
            width: addresses[1],
            color: [addresses[2], addresses[3]],
        });
    }

    let mut paths = Vec::with_capacity(try_u32_usize(bytes, i)?);
    for _ in 0..paths.capacity() {
        let steps = try_u32_usize(bytes, i)?;
        let raw_path = try_n_u32_usize(bytes, i, steps * 2)?;
        paths.push(
            raw_path
                .chunks(2)
                .map(|step| {
                    let (s_type, s_idx) = (step[0], step[1]);
                    let s_type = *STEP_TYPES.get(s_type).ok_or(InvalidStepType)?;
                    Ok((s_type, s_idx))
                })
                .collect::<Result<Path, ParsingError>>()?,
        );
    }

    let mut backgrounds = Vec::with_capacity(try_u32_usize(bytes, i)?);
    for _ in 0..backgrounds.capacity() {
        let triangles = try_u32_usize(bytes, i)?;
        let raw_bg = try_n_u32_usize(bytes, i, triangles)?;
        backgrounds.push(raw_bg);
    }

    let mut rendering_steps = Vec::with_capacity(try_u32_usize(bytes, i)?);
    for _ in 0..rendering_steps.capacity() {
        let clip_or_stroke = try_u32(bytes, i)?;
        let path = try_u32_usize(bytes, i)?;
        let other = try_u32_usize(bytes, i)?;
        rendering_steps.push(match clip_or_stroke {
            0 => RenderingStep::Clip(path, other),
            1 => RenderingStep::Stroke(path, other),
            _ => Err(InvalidRenderingStep)?,
        });
    }

    // names of arguments
    for j in 0..arg_n_len.len() {
        let len = arg_n_len[j];
        if len != 0 {
            let subslice = slice(bytes, i, len)?;
            let arg_name = String::from_utf8(subslice.to_vec()).ok();
            arguments[j].name = Some(arg_name.ok_or(InvalidName)?);
        }
    }

    // names of outputs
    for j in 0..output_n_len.len() {
        let len = output_n_len[j];
        if len != 0 {
            let subslice = slice(bytes, i, len)?;
            let arg_name = String::from_utf8(subslice.to_vec()).ok();
            outputs[j].name = Some(arg_name.ok_or(InvalidName)?);
        }
    }

    Ok(Program {
        arguments,
        instructions,
        outputs,
        arcs,
        cubic_curves,
        quadratic_curves,
        lines,
        triangles,
        strokers,
        paths,
        backgrounds,
        rendering_steps,
    })
}

fn _u32(n: usize) -> u32 {
    n as u32
}

pub fn size(p: &Program) -> usize {
    let mut sz = MAGIC_BYTES.len();
    let mut u32s = 1 + p.arguments.len() * 7;
    u32s += 1 + p.instructions.len() * 4;
    u32s += 1 + p.outputs.len() * 2;
    u32s += 1 + p.triangles.len() * 9;
    u32s += 1 + p.arcs.len() * 3;
    u32s += 1 + p.cubic_curves.len() * 4;
    u32s += 1 + p.quadratic_curves.len() * 3;
    u32s += 1 + p.lines.len() * 2;
    u32s += 1 + p.strokers.len() * 4;
    u32s += 1 + p.rendering_steps.len() * 4;
    u32s += p.paths.iter().fold(1, |a, i| a + 1 + i.len() * 2);
    u32s += p.backgrounds.iter().fold(1, |a, i| a + 1 + i.len());
    for i in &p.arguments {
        if let Some(s) = &i.name {
            sz += s.len();
        }
    }
    for i in &p.outputs {
        if let Some(s) = &i.name {
            sz += s.len();
        }
    }
    sz + size_of::<u32>() * u32s
}

pub fn dump<T: Write>(src: &Program, dst: &mut T) -> IoResult<usize> {
    let mut sz = dst.write(&MAGIC_BYTES)?;

    sz += dst.write(&_u32(src.arguments.len()).to_be_bytes())?;
    for i in &src.arguments {
        sz += dst.write(
            &match &i.name {
                Some(s) => _u32(s.len()),
                _ => 0,
            }
            .to_be_bytes(),
        )?;
        sz += dst.write(&i.value.x.to_be_bytes())?;
        sz += dst.write(&i.value.y.to_be_bytes())?;
        sz += dst.write(&i.range.0.x.to_be_bytes())?;
        sz += dst.write(&i.range.1.x.to_be_bytes())?;
        sz += dst.write(&i.range.0.y.to_be_bytes())?;
        sz += dst.write(&i.range.1.y.to_be_bytes())?;
    }

    sz += dst.write(&_u32(src.instructions.len()).to_be_bytes())?;
    for i in &src.instructions {
        sz += dst.write(&i.operation.opcode().to_be_bytes())?;
        sz += dst.write(&i.operands[0].to_be_bytes())?;
        sz += dst.write(&i.operands[1].to_be_bytes())?;
        sz += dst.write(&i.operands[2].to_be_bytes())?;
    }

    sz += dst.write(&_u32(src.outputs.len()).to_be_bytes())?;
    for i in &src.outputs {
        sz += dst.write(
            &match &i.name {
                Some(s) => _u32(s.len()),
                _ => 0,
            }
            .to_be_bytes(),
        )?;
        sz += dst.write(&i.address.to_be_bytes())?;
    }

    sz += dst.write(&_u32(src.triangles.len()).to_be_bytes())?;
    for i in &src.triangles {
        sz += dst.write(&i.points[0].to_be_bytes())?;
        sz += dst.write(&i.points[1].to_be_bytes())?;
        sz += dst.write(&i.points[2].to_be_bytes())?;
        sz += dst.write(&i.colors[0][0].to_be_bytes())?;
        sz += dst.write(&i.colors[0][1].to_be_bytes())?;
        sz += dst.write(&i.colors[1][0].to_be_bytes())?;
        sz += dst.write(&i.colors[1][1].to_be_bytes())?;
        sz += dst.write(&i.colors[2][0].to_be_bytes())?;
        sz += dst.write(&i.colors[2][1].to_be_bytes())?;
    }

    sz += dst.write(&_u32(src.arcs.len()).to_be_bytes())?;
    for i in &src.arcs {
        sz += dst.write(&i.center.to_be_bytes())?;
        sz += dst.write(&i.angular_range.to_be_bytes())?;
        sz += dst.write(&i.radii.to_be_bytes())?;
    }

    sz += dst.write(&_u32(src.cubic_curves.len()).to_be_bytes())?;
    for i in &src.cubic_curves {
        sz += dst.write(&i.points[0].to_be_bytes())?;
        sz += dst.write(&i.points[1].to_be_bytes())?;
        sz += dst.write(&i.points[2].to_be_bytes())?;
        sz += dst.write(&i.points[3].to_be_bytes())?;
    }

    sz += dst.write(&_u32(src.quadratic_curves.len()).to_be_bytes())?;
    for i in &src.quadratic_curves {
        sz += dst.write(&i.points[0].to_be_bytes())?;
        sz += dst.write(&i.points[1].to_be_bytes())?;
        sz += dst.write(&i.points[2].to_be_bytes())?;
    }

    sz += dst.write(&_u32(src.lines.len()).to_be_bytes())?;
    for i in &src.lines {
        sz += dst.write(&i.points[0].to_be_bytes())?;
        sz += dst.write(&i.points[1].to_be_bytes())?;
    }

    sz += dst.write(&_u32(src.strokers.len()).to_be_bytes())?;
    for i in &src.strokers {
        sz += dst.write(&i.pattern.to_be_bytes())?;
        sz += dst.write(&i.width.to_be_bytes())?;
        sz += dst.write(&i.color[0].to_be_bytes())?;
        sz += dst.write(&i.color[1].to_be_bytes())?;
    }

    sz += dst.write(&_u32(src.paths.len()).to_be_bytes())?;
    for i in &src.paths {
        sz += dst.write(&_u32(i.len()).to_be_bytes())?;
        for (step_type, index) in i {
            sz += dst.write(&step_type.as_u32().to_be_bytes())?;
            sz += dst.write(&_u32(*index).to_be_bytes())?;
        }
    }

    sz += dst.write(&_u32(src.backgrounds.len()).to_be_bytes())?;
    for i in &src.backgrounds {
        sz += dst.write(&_u32(i.len()).to_be_bytes())?;
        for index in i {
            sz += dst.write(&_u32(*index).to_be_bytes())?;
        }
    }

    sz += dst.write(&_u32(src.rendering_steps.len()).to_be_bytes())?;
    for i in &src.rendering_steps {
        let (clip_or_stroke, i1, i2) = match i {
            RenderingStep::Clip(p, b) => (0u32, *p, *b),
            RenderingStep::Stroke(p, s) => (1u32, *p, *s),
        };
        sz += dst.write(&clip_or_stroke.to_be_bytes())?;
        sz += dst.write(&_u32(i1).to_be_bytes())?;
        sz += dst.write(&_u32(i2).to_be_bytes())?;
    }

    for i in &src.arguments {
        if let Some(s) = &i.name {
            sz += dst.write(s.as_bytes())?;
        }
    }

    for i in &src.outputs {
        if let Some(s) = &i.name {
            sz += dst.write(s.as_bytes())?;
        }
    }

    Ok(sz)
}

type R<T> = Result<T, ParsingError>;
type V<T> = R<Vec<T>>;

fn slice<'a>(b: &'a [u8], i: &mut usize, len: usize) -> R<&'a [u8]> {
    let pos = *i;
    *i += len;
    match b.get(pos..*i) {
        Some(bytes) => Ok(bytes),
        None => Err(TooShort),
    }
}

fn try_u32<'a>(b: &'a [u8], i: &mut usize) -> R<u32> {
    let u8x4 = slice(b, i, 4)?;
    let bytes: [u8; 4] = u8x4.try_into().unwrap();
    Ok(u32::from_be_bytes(bytes))
}

fn try_u32_addr<'a>(b: &'a [u8], i: &mut usize) -> R<Address> {
    try_u32(b, i).map(|r| r as Address)
}

fn try_u32_usize<'a>(b: &'a [u8], i: &mut usize) -> R<usize> {
    try_u32(b, i).map(|r| r as usize)
}

fn try_f32<'a>(b: &'a [u8], i: &mut usize) -> R<f32> {
    let u8x4 = slice(b, i, 4)?;
    let bytes: [u8; 4] = u8x4.try_into().unwrap();
    Ok(f32::from_be_bytes(bytes))
}

fn try_n_u32<'a>(b: &'a [u8], i: &mut usize, n: usize) -> V<u32> {
    let mut values = Vec::with_capacity(n);
    for _ in 0..n {
        values.push(try_u32(b, i)?);
    }
    Ok(values)
}

fn try_n_u32_addr<'a>(b: &'a [u8], i: &mut usize, n: usize) -> V<u32> {
    try_n_u32(b, i, n).map(|v| v.iter().map(|r| *r as Address).collect())
}

fn try_n_u32_usize<'a>(b: &'a [u8], i: &mut usize, n: usize) -> V<usize> {
    try_n_u32(b, i, n).map(|v| v.iter().map(|r| *r as usize).collect())
}
