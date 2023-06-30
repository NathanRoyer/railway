use std::env::args;
use std::fs::write;
use png::Encoder;
use png::ColorType::Rgba;
use png::BitDepth::Eight;
use railway::*;
use std::time::Instant;
use rgb::FromSlice;

fn main() {
	let prefix = args().last().unwrap();
	let rwy_name = format!("{}.rwy", &prefix);
	let png_name = format!("{}.png", &prefix);

	let railway = std::fs::read(&rwy_name).unwrap();
	let mut p = NaiveRenderer::parse(railway.into_boxed_slice()).unwrap();
	let (w, h) = (300, 300);
	p.set_argument("size", computing::Couple::new(w as f32, h as f32)).unwrap();
	p.compute().unwrap();
	let length = w * h;
	let mut canvas :Vec<u8> = vec![0; length * 4];
	let mut mask = vec![0; length];

	let runs = 10;
	let now = Instant::now();
	for _ in 0..runs {
		p.render::<6, 36>(canvas.as_rgba_mut(), &mut mask, w, h, w, true).unwrap();
	}
	println!("rendered {} times in {}ms.", runs, now.elapsed().as_millis());

	let mut png_buf = Vec::new();
	{
		let mut encoder = Encoder::new(&mut png_buf, w as u32, h as u32);
		encoder.set_color(Rgba);
		encoder.set_depth(Eight);
		let mut writer = encoder.write_header().unwrap();
		writer.write_image_data(&canvas).unwrap();
	}
	write(&png_name, &png_buf).unwrap();
}
