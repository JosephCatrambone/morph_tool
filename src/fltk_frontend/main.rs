use fltk::{
	app,
	button::Button,
	draw,
	enums::{Color, ColorDepth, Shortcut},
	frame::Frame,
	image as imagefl,
	menu,
	prelude::*,
	surface::ImageSurface,
	utils,
	window::Window,
};
use image as imagers;
use imageproc;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Mutex, Arc};
use morph_tool::animation_system::Animation;
use morph_tool::image_source::*;

/*
App:
┌───────────────────────┐
│ ┌────┐ ┌────┐ ┌────┐  │
│ │ L  │ │ O  │ │ R  │  │
│ │    │ │    │ │    │  │
│ └────┘ └────┘ └────┘  │
├─Timeline:-────────────┤
│┼┼┼┼┼┼┼┼┼┼┼┼┼┼┼┼┼┼┼┼┼┼┼│
└───────────────────────┘

Since our output image and points are cached by the draw function we don't need to maintain cached images of anything for imminent redraw.
*/

struct PointEditor {
	// UI:
	frame: Frame,
	surface: Rc<RefCell<ImageSurface>>,
	needs_frame_update: bool,
	// Data:
	image_source: Arc<Mutex<dyn FrameProvider>>,
	keypoints: Arc<Mutex<Vec<f32>>>,
}

impl PointEditor {
	pub fn new() -> Self {
		let image_source = Arc::new(Mutex::new(NullImageProvider::new()));
		let keypoints = Arc::new(Mutex::new(vec![]));
		let mut frame = Frame::default().size_of_parent();
		frame.set_color(Color::Black);
		//frame.set_frame()

		let surface = ImageSurface::new(frame.w(), frame.h(), false);
		ImageSurface::push_current(&surface);
		draw::draw_rect_fill(0, 0, frame.w(), frame.h(), Color::White);
		ImageSurface::pop_current();

		// Why not new?  Why ::from?
		let surface = Rc::from(RefCell::from(surface));

		// Setup callbacks:
		frame.draw({
			let image_source = image_source.clone();
			let surf = surface.clone();
			move || {

			}
		});

		Self {
			frame,
			surface,
			image_source,
			keypoints,
		}
	}
}

fn main() {
	let app = app::App::default();
	let mut wind = Window::new(100, 100, 400, 300, "Hello from rust");
	let mut frame = Frame::default_fill();

	// Set up main menu:
	let mut menubar = menu::MenuBar::new(0, 0, 400, 40, "rew");
	menubar.add("File/New\t", Shortcut::None, menu::MenuFlag::Normal, menu_cb);
	menubar.add("Morph/Open Left Frame\t", Shortcut::None, menu::MenuFlag::Normal, |m| {});
	menubar.add("Morph/Open Right Frame\t", Shortcut::None, menu::MenuFlag::Normal, |m| {});

	// Set up drawing area:
	let mut but = Button::new(160, 210, 80, 40, "Click me!");
	wind.make_resizable(true);
	wind.end();
	wind.show();

	frame.draw(move |f| {
		let mut image = imagefl::RgbImage::new(&fb, f.w(), f.h(), ColorDepth::Rgba8)
			.unwrap()
			.to_srgb_image()
			.unwrap()
			.blur(50)
			.unwrap()
			.convert(ColorDepth::Rgb8)
			.unwrap();
		image.draw(f.x(), f.y(), f.width(), f.height());
	});

	//but.set_callback(move |_| frame.set_label("Hello World!")); // the closure capture is mutable borrow to our button
	app.run().unwrap();
	// app.run().unwrap() lets the thread loop run and the app do its thing.
	// while app.wait() {} gives us control over handling messages.
}



fn rs_image_to_fl_image(img: &imagers::DynamicImage) -> Result<imagefl::RgbImage, FltkError> {
	let width = img.width() as usize;
	let height = img.height() as usize;
	imagefl::RgbImage::new(
		if let Some(converted) = img.as_rgb8() {
			converted.as_raw()
		} else {
			img.clone().into_rgb8().as_raw()
		},
		width as i32,
		height as i32,
		ColorDepth::Rgb8
	)
}

fn menu_cb(m: &mut impl MenuExt) {
	if let Some(choice) = m.choice() {
		match choice.as_str() {
			"New\t" => println!("asdf"),
			_ => println!("{}", &choice),
		}
	}
}