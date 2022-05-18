pub mod computing;
pub mod primitive;
pub mod drawing;
pub mod format;

#[cfg(feature = "zeno")]
pub mod zeno_rdr;

#[cfg(test)]
mod tests;

use std::io::Write;
use std::io::Result as IoResult;

pub use computing::Argument;
pub use computing::Output;
pub use computing::Operation;
pub use computing::OPERANDS;
pub use computing::OPERATIONS;
pub use computing::Instruction;
pub use computing::compute;

pub use primitive::Arc;
pub use primitive::CubicCurve;
pub use primitive::QuadraticCurve;
pub use primitive::Line;

pub use drawing::Triangle;
pub use drawing::TriangleIndex;
pub use drawing::StepType;
pub use drawing::STEP_TYPES;
pub use drawing::StepIndex;
pub use drawing::Stroker;
pub use drawing::StrokerIndex;
pub use drawing::Path;
pub use drawing::PathIndex;
pub use drawing::Background;
pub use drawing::BackgroundIndex;
pub use drawing::RenderingStep;

pub use format::ParsingError;
pub use format::parse;
pub use format::dump;
pub use format::size;

pub const RWY_PXF_ARGB8888: u8 = 0;
pub const RWY_PXF_RGBA8888: u8 = 1;

pub type Address = u32;
pub type ColorAddress = [Address; 2];

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
	pub arguments: Vec<Argument>,
	pub instructions: Vec<Instruction>,
	pub outputs: Vec<Output>,
	pub arcs: Vec<Arc>,
	pub cubic_curves: Vec<CubicCurve>,
	pub quadratic_curves: Vec<QuadraticCurve>,
	pub lines: Vec<Line>,
	pub triangles: Vec<Triangle>,
	pub strokers: Vec<Stroker>,
	pub paths: Vec<Path>,
	pub backgrounds: Vec<Background>,
	pub rendering_steps: Vec<RenderingStep>,
}

pub type Float = f32;

#[cfg(not(feature = "zeno"))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Couple {
	pub x: Float,
	pub y: Float,
}

#[cfg(feature = "zeno")]
pub use zeno::Vector as Couple;

pub fn at(stack: &[Couple], i: Address) -> Couple {
	stack[i as usize]
}

impl Program {
	pub fn new()-> Self {
		Self {
			arguments: Vec::new(),
			instructions: Vec::new(),
			outputs: Vec::new(),
			arcs: Vec::new(),
			cubic_curves: Vec::new(),
			quadratic_curves: Vec::new(),
			lines: Vec::new(),
			triangles: Vec::new(),
			strokers: Vec::new(),
			paths: Vec::new(),
			backgrounds: Vec::new(),
			rendering_steps: Vec::new(),
		}
	}

	pub fn parse(bytes: &[u8]) -> Result<Self, ParsingError> {
		let program = parse(bytes)?;
		match program.valid() {
			Some(_) => Ok(program),
			None => Err(ParsingError::InvalidIndex),
		}
	}

	pub fn file_size(&self) -> usize {
		size(self)
	}

	pub fn dump<T: Write>(&self, dst: &mut T) -> IoResult<usize> {
		dump(self, dst)
	}

	pub fn stack_size(&self) -> usize {
		self.arguments.len() + self.instructions.len()
	}

	pub fn create_stack(&self) -> Vec<Couple> {
		let mut stack = Vec::with_capacity(self.stack_size());
		for i in &self.arguments {
			stack.push(i.value);
		}
		stack
	}

	pub fn compute(&self, stack: &mut Vec<Couple>) {
		compute(self, stack);
	}

	/// get the address of this argument's value in stack
	pub fn argument(&self, name: &str) -> Option<Address> {
		for i in 0..self.arguments.len() {
			match &self.arguments[i].name {
				Some(s) if s == name => return Some(i as Address),
				_ => (),
			}
		}
		None
	}

	/// get the address of this output's value in stack
	pub fn output(&self, name: &str) -> Option<Address> {
		for o in &self.outputs {
			match &o.name {
				Some(s) if s == name => return Some(o.address),
				_ => (),
			}
		}
		None
	}
	
	#[cfg(feature = "zeno")]
	pub fn render<const PXF: u8>(&self, stack: &[Couple], dst: &mut [u8],
		mask: &mut [u8], width: usize, height: usize, pitch: usize) {
		zeno_rdr::rdr::<PXF>(self, stack, dst, mask, width, height, pitch);
	}

	/// does not check argument default values (on purpose)
	pub fn valid(&self) -> Option<()> {
		let mut max = self.arguments.len() as Address;
		let qc_max = self.quadratic_curves.len();
		let cc_max = self.cubic_curves.len();
		let c_max = self.backgrounds.len();
		let s_max = self.strokers.len();
		let l_max = self.lines.len();
		let a_max = self.arcs.len();

		for i in &self.instructions {
			inf_s(&i.operands, max)?;
			max += 1;
		}
		for i in &self.outputs {
			inf(i.address, max)?;
		}
		for i in &self.arcs {
			inf_s(&[i.center, i.angular_range, i.radii], max)?;
		}
		for i in &self.cubic_curves {
			inf_s(&i.points, max)?;
		}
		for i in &self.quadratic_curves {
			inf_s(&i.points, max)?;
		}
		for i in &self.lines {
			inf_s(&i.points, max)?;
		}
		for i in &self.triangles {
			inf_s(&i.points, max)?;
			for c in i.colors {
				inf_s(&c, max)?;
			}
		}
		for i in &self.strokers {
			inf_s(&[i.pattern, i.width], max)?;
			inf_s(&i.color, max)?;
		}
		for i in &self.paths {
			for (step_type, index) in i {
				inf(*index, match *step_type {
					StepType::QuadraticCurve => qc_max,
					StepType::CubicCurve => cc_max,
					StepType::Line => l_max,
					StepType::Arc => a_max,
				})?;
			}
		}
		for i in &self.backgrounds {
			let max = self.triangles.len();
			i.iter().map(|idx| inf(*idx, max)).collect::<Option<_>>()?;
		}
		for i in &self.rendering_steps {
			let (i1, i2, max) = match i {
				RenderingStep::Clip  (i1, i2) => (*i1, *i2, c_max),
				RenderingStep::Stroke(i1, i2) => (*i1, *i2, s_max),
			};
			inf(i1, self.paths.len())?;
			inf(i2, max)?;
		}
		Some(())
	}
}

fn inf<T: Ord>(a: T, b: T) -> Option<()> {
	match a < b {
		true => Some(()),
		false => None,
	}
}

fn inf_s<T: Ord + Copy>(a: &[T], b: T) -> Option<()> {
	a.iter().map(|a| inf(*a, b)).collect::<Option<_>>()?;
	Some(())
}

#[cfg(not(feature = "zeno"))]
impl From<(f32, f32)> for Couple {
	fn from(couple: (f32, f32)) -> Self {
		Couple {
			x: couple.0,
			y: couple.1,
		}
	}
}

#[cfg(not(feature = "zeno"))]
impl Couple {
	pub fn new(x: f32, y: f32) -> Self {
		Self::from((x, y))
	}
}

pub const C_ZERO: Couple = Couple {
	x: 0f32,
	y: 0f32
};
