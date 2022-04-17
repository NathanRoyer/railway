use crate::Address;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Arc {
	pub center: Address,
	pub angular_range: Address,
	pub radii: Address,
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
