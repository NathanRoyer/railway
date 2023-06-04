use railway::{*, computing::{*, Operation::*}};
use std::env::args;
use std::fs;

fn arg(name: Option<String>, value: Couple) -> Argument<String> {
	Argument {
		name,
		value,
		range: (value, value),
	}
}

use core::f32::consts::TAU;

fn main() {
	let w = 200.0;
	let h = 200.0;

	let mut arguments = Vec::new();

	let _zero = arguments.len();
	arguments.push(arg(None, C_ZERO));

	let size = arguments.len();
	arguments.push(arg(Some("size".into()), Couple::new(w, h)));

	let top_left_f = arguments.len();
	arguments.push(arg(None, Couple::new(0.05, 0.05)));

	let bottom_right_f = arguments.len();
	arguments.push(arg(None, Couple::new(0.95, 0.95)));

	let contour_rg = arguments.len();
	arguments.push(arg(None, Couple::new(0.5, 0.1)));

	let contour_ba = arguments.len();
	arguments.push(arg(None, Couple::new(0.5, 1.0)));

	let pattern = arguments.len();
	arguments.push(arg(None, Couple::new(100.0, 0.0)));

	let width = arguments.len();
	arguments.push(arg(None, Couple::new(4.0, 0.0)));

	let inverted_rg = arguments.len();
	arguments.push(arg(None, Couple::new(0.1, 0.5)));

	let deltas = arguments.len();
	arguments.push(arg(None, Couple::new(TAU, 0.0)));

	let radius = arguments.len();
	arguments.push(arg(None, Couple::new(0.0, -40.0)));

	let center_f = arguments.len();
	arguments.push(arg(None, Couple::new(0.75, 0.25)));

	let mut instructions = Vec::new();

	let top_left = arguments.len() + instructions.len();
	instructions.push(Instruction::new(Multiply2, size, top_left_f, 0));

	let bottom_right = arguments.len() + instructions.len();
	instructions.push(Instruction::new(Multiply2, size, bottom_right_f, 0));

	let bottom_left = arguments.len() + instructions.len();
	instructions.push(Instruction::new(Select2, top_left, bottom_right, 0));

	let top_right = arguments.len() + instructions.len();
	instructions.push(Instruction::new(Select2, bottom_right, top_left, 0));

	let center = arguments.len() + instructions.len();
	instructions.push(Instruction::new(Multiply2, size, center_f, 0));

	let start_point = arguments.len() + instructions.len();
	instructions.push(Instruction::new(Add2, center, radius, 0));

	let contour = [contour_rg, contour_ba];

	let line_style = Stroker {
		pattern,
		width,
		color: contour,
	};

	let background = vec![
		Triangle {
			points: [top_left, bottom_left, bottom_right],
			colors: [contour, [inverted_rg, contour_ba], contour],
		},
		Triangle {
			points: [top_left, top_right, bottom_right],
			colors: [contour, [inverted_rg, contour_ba], contour],
		},
	];

	let slope = vec![
		PathStep::Line(Line {
			points: [bottom_left, top_left],
		}),
		PathStep::QuadraticCurve(QuadraticCurve {
			points: [top_left, bottom_left, bottom_right],
		}),
		PathStep::Line(Line {
			points: [bottom_right, bottom_left],
		}),
	];

	let disk = vec![PathStep::Arc(Arc {
		start_point,
		center,
		deltas,
	})];

	let rendering_steps = [
		RenderingStep::Clip(&slope, &background),
		RenderingStep::Stroke(&slope, line_style),
		RenderingStep::Clip(&disk, &background),
		RenderingStep::Stroke(&disk, line_style),
	];

	let buffer = serialize(&arguments, &instructions, &[], &rendering_steps);
	let file_name = args().last().unwrap();
	fs::write(file_name, &buffer).unwrap();
}
