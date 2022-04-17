use std::env::args;
use std::fs::write;
use png::Encoder;
use png::ColorType::Rgba;
use png::BitDepth::Eight;
use railway::Program;
use std::time::Instant;

fn main() {
	let prefix = args().last().unwrap();
	let rwy_name = format!("{}.rwy", &prefix);
	let png_name = format!("{}.png", &prefix);

	let railway = std::fs::read(&rwy_name).unwrap();
	let p = Program::parse(&railway).unwrap();
	let mut stack = p.create_stack();
	p.compute(&mut stack);
	let size = p.arguments[p.argument("size").unwrap() as usize].value;
	let size = (size.x as usize, size.y as usize);
	let length = size.0 * size.1;
	let mut canvas = vec![0; length * 4];
	let mut mask = vec![0; length];

	let runs = 1;
	let now = Instant::now();
	for _ in 0..runs {
		p.render(&stack, &mut canvas, &mut mask, size.0);
	}
	println!("rendered {} times in {}ms.", runs, now.elapsed().as_millis());

	let mut png_buf = Vec::new();
	{
		let mut encoder = Encoder::new(&mut png_buf, size.0 as u32, size.1 as u32);
		encoder.set_color(Rgba);
		encoder.set_depth(Eight);
		let mut writer = encoder.write_header().unwrap();
		writer.write_image_data(&canvas).unwrap();
	}
	write(&png_name, &png_buf).unwrap();
}
