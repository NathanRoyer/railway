use crate::Address;
use crate::ColorAddress;

use alloc::vec::Vec;

use StepType::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Stroker {
    pub pattern: Address,
    /// stroke width = stack[w].x + stack[w].y
    pub width: Address,
    pub color: ColorAddress,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Triangle {
    pub points: [Address; 3],
    pub colors: [ColorAddress; 3],
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum StepType {
    Arc,
    CubicCurve,
    QuadraticCurve,
    Line,
}

pub type TriangleIndex = usize;
pub type StepIndex = usize;
pub type Path = Vec<(StepType, StepIndex)>;
pub type PathIndex = usize;
pub type Background = Vec<TriangleIndex>;
pub type BackgroundIndex = usize;
pub type StrokerIndex = usize;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RenderingStep {
    Clip(PathIndex, BackgroundIndex),
    Stroke(PathIndex, StrokerIndex),
}

pub const STEP_TYPES: [StepType; 4] = [Arc, CubicCurve, QuadraticCurve, Line];

impl StepType {
    pub fn as_u32(self) -> u32 {
        match self {
            Arc => 0,
            CubicCurve => 1,
            QuadraticCurve => 2,
            Line => 3,
        }
    }
}
