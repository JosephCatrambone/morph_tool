use fltk::{
	app,
	button::Button,
	enums::ColorDepth,
	frame::Frame,
	image as fltk_image,
	prelude::*,
	utils,
	window::Window,
};

fn main() {
	let app = app::App::default();
	let mut wind = Window::new(100, 100, 400, 300, "Hello from rust");
	let mut frame = Frame::default_fill();
	let mut but = Button::new(160, 210, 80, 40, "Click me!");
	wind.make_resizable(true);
	wind.end();
	wind.show();

	frame.draw(move |f| {
		let mut fb: Vec<u8> = vec![0u8; (f.w() * f.h() * 4) as usize];
		for (iter, pixel) in fb.chunks_exact_mut(4).enumerate() {
			let x = iter % f.w() as usize;
			let y = iter / f.w() as usize;
			let (red, green, blue) = utils::hex2rgb((x ^ y) as u32);
			pixel.copy_from_slice(&[red, green, blue, 255]);
		}
		let mut image = fltk_image::RgbImage::new(&fb, f.w(), f.h(), ColorDepth::Rgba8)
			.unwrap()
			.to_srgb_image()
			.unwrap()
			.blur(50)
			.unwrap()
			.convert(ColorDepth::Rgb8)
			.unwrap();
		image.draw(f.x(), f.y(), f.width(), f.height());
	});

	but.set_callback(move |_| frame.set_label("Hello World!")); // the closure capture is mutable borrow to our button
	app.run().unwrap();
}