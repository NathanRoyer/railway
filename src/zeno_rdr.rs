use crate::StepType::QuadraticCurve;
use crate::RenderingStep::Stroke;
use crate::StepType::CubicCurve;
use crate::RenderingStep::Clip;
use crate::StepType::Line;
use crate::StepType::Arc;
use crate::Program;
use crate::Couple;

use zeno::PathBuilder;
use zeno::Command;
use zeno::Stroke as ZenoStroke;
use zeno::Fill;
use zeno::Mask;

// use core::f32::consts::TAU; // 2*PI
use core::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
	/// points
	p: [Couple; 3],
	// colors
	c: [[Couple; 2]; 3],
	// cached vectors used for fast weighting
	v: [Couple; 2],
}

impl Triangle {
	pub fn new(p: [Couple; 3], c: [[Couple; 2]; 3]) -> Self {
		let v0 = Couple::new(p[1].x - p[0].x, p[1].y - p[0].y);
		let v1 = Couple::new(p[2].x - p[0].x, p[2].y - p[0].y);
		let den = 1.0 / (v0.x * v1.y - v1.x * v0.y);
		let v0 = Couple::new(v0.x * den, v0.y * den);
		let v1 = Couple::new(v1.x * den, v1.y * den);
		Self {
			p,
			c,
			v: [v0, v1],
		}
	}

	pub fn weights(&self, pt: Couple) -> Option<(f32, f32, f32)> {
		let v2_x = pt.x - self.p[0].x;
		let v2_y = pt.y - self.p[0].y;
		let v = v2_x * self.v[1].y - v2_y * self.v[1].x;
		let w = v2_y * self.v[0].x - v2_x * self.v[0].y;
		let u = 1.0 - v - w;
		match u >= 0.0 && v >= 0.0 && w >= 0.0 {
			true => Some((u, v, w)),
			false => None,
		}
	}

	pub fn color_at(&self, pt: Couple) -> Option<[f32; 4]> {
		let (a, b, c) = self.weights(pt)?;
		let m = self.c; // color map
		let cr = m[0][0].x * a + m[1][0].x * b + m[2][0].x * c;
		let cg = m[0][0].y * a + m[1][0].y * b + m[2][0].y * c;
		let cb = m[0][1].x * a + m[1][1].x * b + m[2][1].x * c;
		let ca = m[0][1].y * a + m[1][1].y * b + m[2][1].y * c;
		Some([cr, cg, cb, ca])
	}
}

fn begin(p: &mut Vec<Command>, c: Couple) {
	match p.len() == 0 {
		true => p.move_to(c),
		false => p.line_to(c),
	};
}

/*fn ponder(q: u8, f: f32) -> u8 {
	(q as f32 * f).clamp(u8::MIN as f32, u8::MAX as f32).round() as u8
}*/
fn ponder(a: u8, b: f32, p: u8) -> u8 {
	let p = (p as f32) / 255.0;
	let n = 1.0 - p;
	((a as f32) * n + b * 255.0 * p) as u8
}

pub fn rdr(p: &Program, s: &[Couple], d: &mut [u8], m: &mut [u8], w: usize) {
	let s = |a: u32| s[a as usize];
	let h = d.len() / (w * 4);
	let mut paths = Vec::with_capacity(p.paths.len());
	for path in &p.paths {
		let mut zeno_path = Vec::new();
		for (step_type, index) in path {
			match step_type {
				Arc => {
					let arc = p.arcs[*index];
					let c = s(arc.center);
					let (r_x, r_y) = s(arc.radii).into();
					let (a_x, a_y) = s(arc.angular_range).into();
					let angle = a_y - a_x;
					let rad_d = r_y - r_x;
					let max_r = r_x.max(r_y);
					let half_circle_perimeter = PI * max_r;
					let half_perimeter = half_circle_perimeter * angle.abs() / PI;
					let steps = half_perimeter.round();
					let a_delta = angle / steps;
					let r_delta = rad_d / steps;
					let mut a = a_x;
					let mut r = r_x;
					let p = |a: f32, r: f32| {
						let (y, x) = a.sin_cos();
						Couple::new(c.x + r * x, c.y + r * y)
					};
					begin(&mut zeno_path, p(a, r));
					while (a_x < a_y && a < a_y) || (a_x > a_y && a > a_y) {
						a += a_delta;
						r += r_delta;
						zeno_path.line_to(p(a, r));
					}
				},
				CubicCurve => {
					let [a, b, c, d] = p.cubic_curves[*index].points;
					begin(&mut zeno_path, s(a));
					zeno_path.curve_to(s(b), s(c), s(d));
				},
				QuadraticCurve => {
					let [a, b, c] = p.quadratic_curves[*index].points;
					begin(&mut zeno_path, s(a));
					zeno_path.quad_to(s(b), s(c));
				},
				Line => {
					let [a, b] = p.lines[*index].points;
					begin(&mut zeno_path, s(a));
					zeno_path.line_to(s(b));
				},
			}
		}
		zeno_path.close();
		paths.push(zeno_path);
	}
	for rs in &p.rendering_steps {
		let zeno_path = &paths[*match rs {
			Clip(i, _) => i,
			Stroke(i, _) => i,
		}];
		// println!("path len: {}", zeno_path.len());
		let mut zeno_mask = Mask::new(&zeno_path);
		zeno_mask.size(w as u32, h as u32);
		m.fill(0);
		if let Clip(_, i) = rs {
			let background = &p.backgrounds[*i];
			let mut triangles = Vec::with_capacity(background.len());
			for ti in background {
				let t = p.triangles[*ti];
				let [p0, p1, p2] = t.points;
				let p = [s(p0), s(p1), s(p2)];
				let [[c0, c1], [c2, c3], [c4, c5]] = t.colors;
				let c = [[s(c0), s(c1)], [s(c2), s(c3)], [s(c4), s(c5)]];
				triangles.push(Triangle::new(p, c));
			}
			zeno_mask.style(Fill::NonZero);
			zeno_mask.render_into(m, None);
			let mut j = 0;
			for y in 0..h {
				for x in 0..w {
					let q = m[y * w + x];
					let point = Couple::new(x as f32, y as f32);
					for t in &triangles {
						if let Some(color) = t.color_at(point) {
							let [r, g, b, a] = color;
							d[j + 0] = ponder(d[j + 0], r, q);
							d[j + 1] = ponder(d[j + 1], g, q);
							d[j + 2] = ponder(d[j + 2], b, q);
							d[j + 3] = ponder(d[j + 3], a, q);
							break;
						}
					}
					j += 4;
				}
			}
		}
		if let Stroke(_, i) = rs {
			let stroker = &p.strokers[*i];
			let p = s(stroker.pattern);
			let p = [p.x, p.y];
			let w = s(stroker.width);
			let rg = s(stroker.color[0]);
			let ba = s(stroker.color[1]);
			let mut s = ZenoStroke::new(w.x + w.y);
			s.dash(&p, 0.0);
			zeno_mask.style(s);
			// println!("A");
			zeno_mask.render_into(m, None);
			// println!("B");
			let mut j = 0;
			for q in m.iter() {
				if *q != 0 {
					d[j + 0] = ponder(d[j + 0], rg.x, *q);
					d[j + 1] = ponder(d[j + 1], rg.y, *q);
					d[j + 2] = ponder(d[j + 2], ba.x, *q);
					d[j + 3] = ponder(d[j + 3], ba.y, *q);
				}
				j += 4;
			}
		}
	}
}
