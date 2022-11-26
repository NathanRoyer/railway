use crate::Couple;
use crate::Program;
use crate::RenderingStep::Clip;
use crate::RenderingStep::Stroke;
use crate::StepType::Arc;
use crate::StepType::CubicCurve;
use crate::StepType::Line;
use crate::StepType::QuadraticCurve;
use crate::RWY_PXF_ARGB8888;
use crate::RWY_PXF_RGBA8888;
use crate::Float;

use wizdraw::push_cubic_bezier_segments;
use wizdraw::stroke;
use wizdraw::fill;

use vek::bezier::CubicBezier2;
use vek::bezier::QuadraticBezier2;
use vek::vec::Vec2;
use vek::num_traits::real::Real;

use core::f32::consts::PI;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy)]
pub struct Triangle<C> {
    /// points
    p: [Couple; 3],
    // colors
    c: [[C; 4]; 3],
    // cached vectors used for fast weighting
    v: [Couple; 2],
}

impl<C: Real> Triangle<C> {
    pub fn new(p: [Couple; 3], c: [[C; 4]; 3]) -> Self {
        let v0 = Couple::new(p[1].x - p[0].x, p[1].y - p[0].y);
        let v1 = Couple::new(p[2].x - p[0].x, p[2].y - p[0].y);
        let den = 1.0 / (v0.x * v1.y - v1.x * v0.y);
        let v0 = Couple::new(v0.x * den, v0.y * den);
        let v1 = Couple::new(v1.x * den, v1.y * den);
        Self { p, c, v: [v0, v1] }
    }

    pub fn weights(&self, pt: Couple) -> Option<(C, C, C)> {
        let v2_x = pt.x - self.p[0].x;
        let v2_y = pt.y - self.p[0].y;
        let v = v2_x * self.v[1].y - v2_y * self.v[1].x;
        let w = v2_y * self.v[0].x - v2_x * self.v[0].y;
        let u = 1.0 - v - w;
        match u.is_sign_positive() && v.is_sign_positive() && w.is_sign_positive() {
            true => Some((
                C::from(u).unwrap(),
                C::from(v).unwrap(),
                C::from(w).unwrap(),
            )),
            false => None,
        }
    }

    pub fn color_at(&self, pt: Couple) -> Option<[u8; 4]> {
        let (a, b, c) = self.weights(pt)?;
        let m = self.c; // color map
        let cr = m[0][0] * a + m[1][0] * b + m[2][0] * c;
        let cg = m[0][1] * a + m[1][1] * b + m[2][1] * c;
        let cb = m[0][2] * a + m[1][2] * b + m[2][2] * c;
        let ca = m[0][3] * a + m[1][3] * b + m[2][3] * c;
        let cr = cr.to_u8().unwrap();
        let cg = cg.to_u8().unwrap();
        let cb = cb.to_u8().unwrap();
        let ca = ca.to_u8().unwrap();
        Some([cr, cg, cb, ca])
    }
}

/*fn ponder(a: u8, b: Float, p: u8) -> u8 {
    let p = (p as Float) / 255.0;
    let n = 1.0 - p;
    ((a as Float) * n + b * 255.0 * p) as u8
}*/

fn ponder(a: u8, b: u8, p: u8) -> u8 {
    let p1024 = ((p as u32) * 1024) / 255;
    let n1024 = 1024 - p1024;
    (((a as u32) * n1024 + (b as u32) * p1024) / 1024) as u8
}

pub fn rdr<const PXF: u8, const SSAA: usize>(
    p: &Program,
    stack: &[Couple],
    dst: &mut [u8],
    mask: &mut [u8],
    w: usize,
    h: usize,
    pitch: usize,
) {
    {
        let mut i = 0;
        let px_width = w * 4;
        for _ in 0..h {
            dst[i..][..px_width].fill(0);
            i += px_width + pitch;
        }
    }
    let channels = match PXF {
        RWY_PXF_ARGB8888 => (3, 0, 1, 2),
        RWY_PXF_RGBA8888 => (0, 1, 2, 3),
        _ => unreachable!(),
    };
    let size = Vec2::from((w, h));
    let s = |a: u32| stack[a as usize];
    let row = w * 4;
    let mut paths = Vec::with_capacity(p.paths.len());
    for path in &p.paths {
        let mut wizdraw_path = Vec::<Couple>::new();
        for (step_type, index) in path {
            match step_type {
                Arc => {
                    let arc = p.arcs[*index];
                    let c = s(arc.center);
                    let (r_x, r_y) = s(arc.radii).into_tuple();
                    let (a_x, a_y) = s(arc.angular_range).into_tuple();
                    let angle = a_y - a_x;
                    let rad_d = r_y - r_x;
                    let max_r = r_x.max(r_y);
                    let half_perimeter = PI * max_r * angle.abs() / PI;
                    let steps = half_perimeter.round();
                    let a_delta = angle / steps;
                    let r_delta = rad_d / steps;
                    let mut a = a_x;
                    let mut r = r_x;
                    let p = |a: Float, r: Float| {
                        let (y, x) = a.sin_cos();
                        Couple::new(c.x + r * x, c.y + r * y)
                    };
                    wizdraw_path.push(p(a, r));
                    while (a_x < a_y && a < a_y) || (a_x > a_y && a > a_y) {
                        a += a_delta;
                        r += r_delta;
                        wizdraw_path.push(p(a, r));
                    }
                }
                CubicCurve => {
                    let [a, b, c, d] = p.cubic_curves[*index].points;
                    let curve = CubicBezier2 {
                        start: s(a),
                        ctrl0: s(b),
                        ctrl1: s(c),
                        end: s(d),
                    };
                    push_cubic_bezier_segments::<_, 8>(&curve, 0.6, &mut wizdraw_path);
                }
                QuadraticCurve => {
                    let [a, b, c] = p.quadratic_curves[*index].points;
                    let curve = QuadraticBezier2 {
                        start: s(a),
                        ctrl: s(b),
                        end: s(c),
                    };
                    push_cubic_bezier_segments::<_, 8>(&curve.into_cubic(), 0.6, &mut wizdraw_path);
                }
                Line => {
                    let [a, b] = p.lines[*index].points;
                    wizdraw_path.push(s(a));
                    wizdraw_path.push(s(b));
                }
            }
        }
        if wizdraw_path.first().is_some() {
            wizdraw_path.push(wizdraw_path[0]);
        }
        paths.push(wizdraw_path);
    }
    for rs in &p.rendering_steps {
        let wizdraw_path = &paths[*match rs {
            Clip(i, _) => i,
            Stroke(i, _) => i,
        }];
        
        mask.fill(0);
        if let Clip(_, i) = rs {
            let background = &p.backgrounds[*i];
            let mut triangles = Vec::with_capacity(background.len());
            for ti in background {
                let t = p.triangles[*ti];
                let [p0, p1, p2] = t.points;
                let p = [s(p0), s(p1), s(p2)];
                let [[c0, c1], [c2, c3], [c4, c5]] = t.colors;
                let p1_c = [s(c0).x * 255.0, s(c0).y * 255.0, s(c1).x * 255.0, s(c1).y * 255.0];
                let p2_c = [s(c2).x * 255.0, s(c2).y * 255.0, s(c3).x * 255.0, s(c3).y * 255.0];
                let p3_c = [s(c4).x * 255.0, s(c4).y * 255.0, s(c5).x * 255.0, s(c5).y * 255.0];
                triangles.push(Triangle::new(p, [p1_c, p2_c, p3_c]));
            }
            fill::<_, SSAA>(&wizdraw_path, mask, size);
            let mut j = 0;
            let mut k = 0;
            let mut q_i = 0;
            for y in 0..h {
                for x in 0..w {
                    let q = mask[q_i];
                    if q != 0 {
                        let point = Couple::new(x as Float, y as Float);
                        for t in &triangles {
                            if let Some(color) = t.color_at(point) {
                                let [r, g, b, a] = color;
                                dst[k + j + channels.0] = ponder(dst[k + j + channels.0], r, q);
                                dst[k + j + channels.1] = ponder(dst[k + j + channels.1], g, q);
                                dst[k + j + channels.2] = ponder(dst[k + j + channels.2], b, q);
                                dst[k + j + channels.3] = ponder(dst[k + j + channels.3], a, q);
                                break;
                            }
                        }
                    }
                    j += 4;
                    if j == row {
                        j = 0;
                        k += row + pitch;
                    }
                    q_i += 1;
                }
            }
        }
        if let Stroke(_, i) = rs {
            let stroker = &p.strokers[*i];
            let p = s(stroker.pattern);
            let _p = [p.x, p.y];
            let w = s(stroker.width);
            stroke::<_, SSAA>(&wizdraw_path, mask, size, w.x + w.y);
            let rg = s(stroker.color[0]);
            let ba = s(stroker.color[1]);
            let [r, g, b, a] = [(rg.x * 255.0) as u8, (rg.y * 255.0) as u8, (ba.x * 255.0) as u8, (ba.y * 255.0) as u8];
            let mut j = 0;
            let mut k = 0;
            for q in mask.iter() {
                let q = *q;
                if q != 0 {
                    dst[k + j + channels.0] = ponder(dst[k + j + channels.0], r, q);
                    dst[k + j + channels.1] = ponder(dst[k + j + channels.1], g, q);
                    dst[k + j + channels.2] = ponder(dst[k + j + channels.2], b, q);
                    dst[k + j + channels.3] = ponder(dst[k + j + channels.3], a, q);
                }
                j += 4;
                if j == row {
                    j = 0;
                    k += row + pitch;
                }
            }
        }
    }
}
