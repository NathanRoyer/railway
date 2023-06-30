use crate::computing::Couple;
use crate::computing::ParsingResult;
use crate::computing::SerializedProgram;
use crate::computing::PathStep;
use crate::computing::RawBackground;
use crate::computing::RawRenderingStep::Clip;
use crate::computing::RawRenderingStep::Stroke;
use crate::computing::Float;
use crate::computing::C_ZERO;

use wizdraw::push_cubic_bezier_segments;
use wizdraw::stroke;
use wizdraw::fill;

use vek::bezier::CubicBezier2;
use vek::bezier::QuadraticBezier2;
use vek::vec::Vec2;

#[allow(unused_imports)]
use vek::num_traits::real::Real;

use rgb::{RGBA, RGBA8, ComponentMap};

use core::f32::consts::FRAC_PI_2;
use alloc::{vec, vec::Vec, boxed::Box};

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    /// points
    p: [Couple; 3],
    // cached vectors used for fast weighting
    v: [Couple; 2],
}

impl Triangle {
    pub const fn invalid() -> Self {
        Self { p: [C_ZERO; 3], v: [C_ZERO; 2] }
    }

    pub fn new(p: [Couple; 3]) -> Self {
        let v0 = Couple::new(p[1].x - p[0].x, p[1].y - p[0].y);
        let v1 = Couple::new(p[2].x - p[0].x, p[2].y - p[0].y);
        let den = 1.0 / (v0.x * v1.y - v1.x * v0.y);
        let v0 = Couple::new(v0.x * den, v0.y * den);
        let v1 = Couple::new(v1.x * den, v1.y * den);
        Self { p, v: [v0, v1] }
    }

    pub fn weights(&self, pt: Couple) -> Option<(Float, Float, Float)> {
        let v2_x = pt.x - self.p[0].x;
        let v2_y = pt.y - self.p[0].y;
        let v = v2_x * self.v[1].y - v2_y * self.v[1].x;
        let w = v2_y * self.v[0].x - v2_x * self.v[0].y;
        let u = 1.0 - v - w;
        match u.is_sign_positive() && v.is_sign_positive() && w.is_sign_positive() {
            true => Some((u, v, w)),
            false => None,
        }
    }

    pub fn color_at(weights: (Float, Float, Float), color_map: [RGBA<Float>; 3]) -> RGBA8 {
        let (a, b, c) = weights;
        let m = color_map;
        let cr = m[0].r * a + m[1].r * b + m[2].r * c;
        let cg = m[0].g * a + m[1].g * b + m[2].g * c;
        let cb = m[0].b * a + m[1].b * b + m[2].b * c;
        let ca = m[0].a * a + m[1].a * b + m[2].a * c;
        [cr as u8, cg as u8, cb as u8, ca as u8].into()
    }
}

pub struct NaiveRenderer<T> {
    program: SerializedProgram<T>,
    stack: Box<[Couple]>,
    stack_changes: Box<[bool]>,
    flat_paths: Box<[Vec<Couple>]>,
    triangles: Box<[Triangle]>,
    triangle_colors: Box<[([RGBA<Float>; 3], bool)]>,
}

impl<T: AsRef<[u8]>> NaiveRenderer<T> {
    pub fn new(program: SerializedProgram<T>) -> ParsingResult<Self> {
        let path_count = program.paths();
        let triangle_count = program.triangles();
        let stack_size = program.stack_size();
        let mut stack = vec![C_ZERO; stack_size].into_boxed_slice();

        let arg_count = program.arguments();
        for i in 0..arg_count {
            stack[i] = program.argument(i)?.value;
        }

        Ok(Self {
            program,
            stack,
            stack_changes: vec![true; stack_size].into_boxed_slice(),
            flat_paths: vec![Vec::new(); path_count].into_boxed_slice(),
            triangles: vec![Triangle::invalid(); triangle_count].into_boxed_slice(),
            triangle_colors: vec![([RGBA::default(); 3], false); triangle_count].into_boxed_slice(),
        })
    }

    pub fn log_stack(&self) -> ParsingResult<()> {
        log::info!(    "| INDEX |   ORIGIN   |   X   |   Y   |");

        let arguments = self.program.arguments();
        for i in 0..arguments {
            let stack_index = i;
            let (x, y) = self.stack[stack_index].into_tuple();
            log::info!("| {:^5} |  Argument  | {:^5} | {:^5} |", stack_index, x, y);
        }

        for i in 0..self.program.instructions() {
            let stack_index = arguments + i;
            let (x, y) = self.stack[stack_index].into_tuple();
            let instruction = self.program.instruction(i)?.operation.as_text();
            log::info!("| {:^5} | {:^10} | {:^5} | {:^5} |", stack_index, instruction, x, y);
        }

        Ok(())
    }

    pub fn parse(bytes: T) -> ParsingResult<Self> {
        Self::new(SerializedProgram::new(bytes)?)
    }

    pub fn get_program(&self) -> &SerializedProgram<T> {
        &self.program
    }

    pub fn get_argument(&mut self, name: &str) -> ParsingResult<Option<Couple>> {
        let arg_count = self.program.arguments();
        let mut position = None;

        for i in 0..arg_count {
            let arg = self.program.argument(i)?;
            if arg.name == Some(name) {
                position = Some(i);
                break;
            }
        }

        Ok(match position {
            None => None,
            Some(p) => Some(self.stack[p]),
        })
    }

    pub fn set_argument(&mut self, name: &str, value: Couple) -> ParsingResult<()> {
        let arg_count = self.program.arguments();

        for i in 0..arg_count {
            let arg = self.program.argument(i)?;
            if arg.name == Some(name) {
                if self.stack[i] != value {
                    self.stack_changes[i] = true;
                    self.stack[i] = value;
                }
                break;
            }
        }

        Ok(())
    }

    pub fn output(&mut self, name: &str) -> ParsingResult<Option<Couple>> {
        let output_count = self.program.outputs();
        let mut position = None;

        for i in 0..output_count {
            let output = self.program.output(i)?;
            if output.name == Some(name) {
                position = Some(i);
                break;
            }
        }

        Ok(match position {
            None => None,
            Some(p) => Some(self.stack[p]),
        })
    }

    pub fn compute(&mut self) -> ParsingResult<()> {
        self.program.compute(&mut self.stack, Some(&mut self.stack_changes))
    }

    pub fn render<const SSAA: usize, const SSAA_SQ: usize>(
        &mut self,
        dst: &mut [RGBA8],
        mask: &mut [u8],
        w: usize,
        h: usize,
        stride: usize,
        alpha_blend: bool,
    ) -> ParsingResult<()> {
        let mask_size = Vec2::new(w, h);

        // clear the rectangle
        if true {
            let mut i = 0;
            for _ in 0..h {
                dst[i..][..w].fill(RGBA8::new(0, 0, 0, 0));
                i += stride;
            }
        }

        // update flattened paths
        let path_count = self.program.paths();
        for p in 0..path_count {
            let mut was_updated = false;
            for step in self.program.path(p)? {
                match step? {
                    PathStep::Arc(arc) => {
                        let a = self.stack_changes[arc.start_point];
                        let c = self.stack_changes[arc.center];
                        let r = self.stack_changes[arc.deltas];
                        was_updated |= c | a | r;
                    }
                    PathStep::CubicCurve(curve) => {
                        let [a, b, c, d] = curve.points;
                        let a = self.stack_changes[a];
                        let b = self.stack_changes[b];
                        let c = self.stack_changes[c];
                        let d = self.stack_changes[d];
                        was_updated |= a | b | c | d;
                    }
                    PathStep::QuadraticCurve(curve) => {
                        let [a, b, c] = curve.points;
                        let a = self.stack_changes[a];
                        let b = self.stack_changes[b];
                        let c = self.stack_changes[c];
                        was_updated |= a | b | c;
                    }
                    PathStep::Line(line) => {
                        let [a, b] = line.points;
                        let a = self.stack_changes[a];
                        let b = self.stack_changes[b];
                        was_updated |= a | b;
                    }
                }
                if was_updated {
                    break;
                }
            }

            if !was_updated {
                continue;
            }

            let mut flat = &mut self.flat_paths[p];
            flat.clear();
            for step in self.program.path(p)? {
                match step? {
                    PathStep::Arc(arc) => {
                        let mut start = self.stack[arc.start_point];
                        let center = self.stack[arc.center];
                        let (mut d_a, mut d_r) = self.stack[arc.deltas].into_tuple();

                        // called for each line covering at most 90°:
                        let mut process = |d_a: f32, d_r: f32, start: Couple| {
                            // uses https://stackoverflow.com/a/44829356
                            // question link: https://stackoverflow.com/questions/734076

                            let cs = start - center;
                            let cs_a = (-cs.y).atan2(cs.x);
                            let (my, x) = (cs_a + d_a).sin_cos();
                            let end = center + (cs.magnitude() + d_r) * Couple::new(x, -my);
                            let ce = end - center;

                            let q1 = cs.x * cs.x + cs.y * cs.y;
                            let q2 = q1 + cs.x * ce.x + cs.y * ce.y;
                            let k2 = (4.0 / 3.0) * ((2.0 * q1 * q2).sqrt() - q2) / (cs.x * ce.y - cs.y * ce.x);

                            let ctrl0_x = center.x + cs.x - k2 * cs.y;
                            let ctrl0_y = center.y + cs.y + k2 * cs.x;
                            let ctrl1_x = center.x + ce.x + k2 * ce.y;
                            let ctrl1_y = center.y + ce.y - k2 * ce.x;

                            let curve = CubicBezier2 {
                                start,
                                ctrl0: (ctrl0_x, ctrl0_y).into(),
                                ctrl1: (ctrl1_x, ctrl1_y).into(),
                                end,
                            };

                            push_cubic_bezier_segments::<8>(&curve, 0.4, &mut flat);

                            end
                        };

                        while d_a.abs() > FRAC_PI_2 {
                            let tmp_d_a = d_a.signum() * FRAC_PI_2;
                            let factor = tmp_d_a / d_a;
                            let tmp_d_r = factor * d_r;

                            start = process(tmp_d_a, tmp_d_r, start);

                            d_a -= tmp_d_a;
                            d_r -= tmp_d_r;
                        }

                        process(d_a, d_r, start);
                    }
                    PathStep::CubicCurve(curve) => {
                        let [a, b, c, d] = curve.points;
                        let curve = CubicBezier2 {
                            start: self.stack[a],
                            ctrl0: self.stack[b],
                            ctrl1: self.stack[c],
                            end: self.stack[d],
                        };
                        push_cubic_bezier_segments::<8>(&curve, 0.6, &mut flat);
                    }
                    PathStep::QuadraticCurve(curve) => {
                        let [a, b, c] = curve.points;
                        let curve = QuadraticBezier2 {
                            start: self.stack[a],
                            ctrl: self.stack[b],
                            end: self.stack[c],
                        };
                        push_cubic_bezier_segments::<8>(&curve.into_cubic(), 0.6, &mut flat);
                    }
                    PathStep::Line(line) => {
                        let [a, b] = line.points;
                        flat.push(self.stack[a]);
                        flat.push(self.stack[b]);
                    }
                }
            }
            if flat.first().is_some() {
                flat.push(flat[0]);
            }
        }

        // update triangles
        let triangle_count = self.program.triangles();
        for t in 0..triangle_count {
            let triangle = self.program.triangle(t)?;
            let pos_changed = triangle.points.iter().find(|p| self.stack_changes[**p]).is_some();
            let colors_changed = triangle.colors.iter().flatten().find(|p| self.stack_changes[**p]).is_some();

            if pos_changed {
                let [p0, p1, p2] = triangle.points;
                self.triangles[t] = Triangle::new([
                    self.stack[p0],
                    self.stack[p1],
                    self.stack[p2],
                ]);
            }

            if colors_changed {
                let c = triangle.colors;
                let p1c = color(self.stack[c[0][0]], self.stack[c[0][1]]);
                let p2c = color(self.stack[c[1][0]], self.stack[c[1][1]]);
                let p3c = color(self.stack[c[2][0]], self.stack[c[2][1]]);
                self.triangle_colors[t] = ([p1c, p2c, p3c], p1c == p2c && p1c == p3c)
            }
        }

        self.stack_changes.fill(false);

        let rendering_step_count = self.program.rendering_steps();
        for r in 0..rendering_step_count {
            let rendering_step = self.program.raw_rendering_step(r)?;

            let path_index = match rendering_step {
                Clip(i, _) => i,
                Stroke(i, _) => i,
            };
            let flat_path = &self.flat_paths[path_index];
            
            mask.fill(0);
            if let Clip(_, i) = rendering_step {
                fill::<SSAA, SSAA_SQ>(&flat_path, mask, mask_size);

                let RawBackground {
                    triangle_index_offset: offset,
                    stop_before,
                } = self.program.raw_background(i)?;

                let mut mask = mask.iter();
                let mut line = 0;
                for y in 0..h {
                    for x in 0..w {
                        let q = *mask.next().unwrap();
                        if q != 0 {
                            let point = Couple::new(x as Float, y as Float);
                            for t in offset..stop_before {
                                let triangle_index = self.program.triangle_index(t)?;
                                let triangle = self.triangles[triangle_index];
                                let (colors, solid) = self.triangle_colors[triangle_index];
                                if let Some(weights) = triangle.weights(point) {
                                    let color = match solid {
                                        true => colors[0].map(|float| float as u8),
                                        false => Triangle::color_at(weights, colors),
                                    };

                                    blend_pixel(&mut dst[line + x], color, q, alpha_blend);
                                }
                            }
                        }
                    }
                    line += stride;
                }
            } else if let Stroke(_, i) = rendering_step {
                let stroker = self.program.stroker(i)?;

                let p = self.stack[stroker.pattern];
                let _p = [p.x, p.y];
                let stroke_width = self.stack[stroker.width];
                stroke::<SSAA>(&flat_path, mask, mask_size, stroke_width.x + stroke_width.y);

                let color = color(self.stack[stroker.color[0]], self.stack[stroker.color[1]]);
                let color = color.map(|float| float as u8);

                let mut mask = mask.iter();
                let mut line = 0;
                for _ in 0..h {
                    for x in 0..w {
                        let q = *mask.next().unwrap();
                        if q != 0 {
                            blend_pixel(&mut dst[line + x], color, q, alpha_blend);
                        }
                    }
                    line += stride;
                }
            }
        }

        Ok(())
    }
}

fn color(rg: Couple, ba: Couple) -> RGBA<f32> {
    RGBA::new(rg.x * 255.0, rg.y * 255.0, ba.x * 255.0, ba.y * 255.0)
}

#[inline(always)]
pub fn blend_pixel(dst_pixel: &mut RGBA8, src_pixel: RGBA8, mask_alpha: u8, alpha_blend_dst: bool) {
    if src_pixel.a == 255 && mask_alpha == 255 {
        let for_each = |src, dst: &mut _| *dst = src;

        for_each(src_pixel.r, &mut dst_pixel.r);
        for_each(src_pixel.g, &mut dst_pixel.g);
        for_each(src_pixel.b, &mut dst_pixel.b);
        for_each(src_pixel.a, &mut dst_pixel.a);
    } else {
        let src_alpha = ((src_pixel.a as u32) * (mask_alpha as u32)) / 255;
        let u8_max = u8::MAX as u32;
        let dst_alpha = u8_max - src_alpha;

        if alpha_blend_dst {
            let for_each = |src, dst: &mut _| {
                let src_scaled = (src as u32) * src_alpha;
                let dst_scaled = (*dst as u32) * dst_alpha;
                *dst = ((src_scaled + dst_scaled) / u8_max) as u8;
            };

            for_each(src_pixel.r, &mut dst_pixel.r);
            for_each(src_pixel.g, &mut dst_pixel.g);
            for_each(src_pixel.b, &mut dst_pixel.b);
            for_each(src_pixel.a, &mut dst_pixel.a);
        } else {
            let for_each = |src, dst: &mut _| {
                *dst = ((src as u32 * src_alpha) / u8_max) as u8;
            };

            for_each(src_pixel.r, &mut dst_pixel.r);
            for_each(src_pixel.g, &mut dst_pixel.g);
            for_each(src_pixel.b, &mut dst_pixel.b);
            for_each(src_pixel.a, &mut dst_pixel.a);
        }
    };
}
