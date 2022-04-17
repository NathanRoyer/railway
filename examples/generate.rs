use railway::*;
use std::env::args;
use std::fs;

fn arg(name: Option<String>, value: Couple) -> Argument {
	Argument {
		name,
		value,
		range: (value, value),
	}
}

use core::f32::consts::TAU;

fn main() {
	let mut p = Program::new();
	let w = 200.0;
	let h = 200.0;

	// top left = 0
	p.arguments.push(arg(None, C_ZERO));
	// size = 1
	p.arguments.push(arg(Some("size".into()), Couple::new(w, h)));
	// top left = 2
	p.arguments.push(arg(None, Couple::new(0.05 * w, 0.05 * h)));
	// bottom right = 3
	p.arguments.push(arg(None, Couple::new(0.95 * w, 0.95 * h)));
	// bottom left = 4
	p.arguments.push(arg(None, Couple::new(0.05 * w, 0.95 * h)));
	// top right = 5
	p.arguments.push(arg(None, Couple::new(0.95 * w, 0.05 * h)));
	// contour = 6 & 7
	p.arguments.push(arg(None, Couple::new(0.5, 0.1)));
	p.arguments.push(arg(None, Couple::new(0.5, 1.0)));
	// pattern = 8
	p.arguments.push(arg(None, Couple::new(100.0, 0.0)));
	// width = 9
	p.arguments.push(arg(None, Couple::new(4.0, 0.0)));
	// inverted rg = 10
	p.arguments.push(arg(None, Couple::new(0.1, 0.5)));
	// half circle angles = 11
	p.arguments.push(arg(None, Couple::new(0.0, TAU)));
	// radii = 12
	p.arguments.push(arg(None, Couple::new(40.0, 40.0)));
	// center = 13
	p.arguments.push(arg(None, Couple::new(0.75 * w, 0.25 * h)));

	p.strokers.push(Stroker {
		pattern: 8,
		width: 9,
		color: [6, 7],
	});

	p.lines.push(Line {
		points: [4, 2]
	});

	p.quadratic_curves.push(QuadraticCurve {
		points: [2, 4, 3]
	});

	p.lines.push(Line {
		points: [3, 4]
	});

	p.arcs.push(Arc {
		center: 13,
		angular_range: 11,
		radii: 12,
	});

	p.triangles.push(Triangle {
		points: [2, 4, 3],
		colors: [[6, 7], [10, 7], [6, 7]],
	});

	p.triangles.push(Triangle {
		points: [2, 5, 3],
		colors: [[6, 7], [10, 7], [6, 7]],
	});

	let l = StepType::Line;
	let c = StepType::QuadraticCurve;
	let a = StepType::Arc;
	p.paths.push(vec![(l, 0), (c, 0), (l, 1)]);
	p.paths.push(vec![(a, 0)]);
	p.backgrounds.push(vec![0, 1]);

	p.rendering_steps.push(RenderingStep::Clip(0, 0));
	p.rendering_steps.push(RenderingStep::Stroke(0, 0));
	p.rendering_steps.push(RenderingStep::Clip(1, 0));
	p.rendering_steps.push(RenderingStep::Stroke(1, 0));

	let mut buffer = Vec::with_capacity(p.file_size());
	p.dump(&mut buffer).unwrap();
	let file_name = args().last().unwrap();
	fs::write(file_name, &buffer).unwrap();
}
