use fltk::{
	app,
	button::Button,
	draw,
	enums::{Color, ColorDepth, Shortcut},
	frame::Frame,
	group,
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
use std::ops::DerefMut;
use std::sync::{Mutex, Arc};
use ndarray::AssignElem;
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

const MENUBAR_HEIGHT: i32 = 24;

struct PointEditor {
	// UI:
	frame: Frame,
	cached_image: Arc<Mutex<imagefl::RgbImage>>,
	// Data:
	image_source: Arc<Mutex<Box<dyn FrameProvider>>>,
	cached_keypoints: Arc<Mutex<Vec<f32>>>,
}

impl PointEditor {
	pub fn new() -> Self {
		let raw_provider = NullImageProvider::new(None);
		let image_source: Arc<Mutex<Box<dyn FrameProvider>>> = Arc::new(Mutex::new(Box::new(raw_provider)));
		let cached_image = Arc::new(Mutex::new(rs_image_to_fl_image(image_source.lock().unwrap().get_frame(0)).unwrap()));
		let keypoints = Arc::new(Mutex::new(vec![]));
		let mut frame = Frame::default().size_of_parent();
		frame.set_color(Color::Black);
		//frame.set_frame()

		/*
		// surface: Rc<RefCell<ImageSurface>>,
		let surface = ImageSurface::new(frame.w(), frame.h(), false);
		ImageSurface::push_current(&surface);
		draw::draw_rect_fill(0, 0, frame.w(), frame.h(), Color::White);
		ImageSurface::pop_current();
		// Why not new?  Why ::from?
		let surface = Rc::from(RefCell::from(surface));
		*/

		// Setup callbacks:
		frame.draw({
			let points = keypoints.clone();
			let image = cached_image.clone();
			//let surf = surface.clone();
			move |f| {
				//let surf = surf.borrow();
				//let mut img = surf.image().unwrap();
				//img.draw(f.x(), f.y(), f.w(), f.h());
				let mut img = image.lock().unwrap();
				img.draw(f.x(), f.y(), f.w(), f.h());

			}
		});

		Self {
			frame,
			cached_image,
			image_source,
			cached_keypoints: keypoints,
		}
	}

	fn load_frame_source(&mut self) {
		if let Some(src) = open_image_source() {
			if let Ok(mut src_lock) = self.image_source.lock() {
				*src_lock.deref_mut() = src;
			}
		}
		let mut img = self.cached_image.lock().unwrap();
		*img = rs_image_to_fl_image(self.image_source.lock().unwrap().get_frame(0)).unwrap();
	}
}

fn main() {
	let app = app::App::default();
	let mut wind = Window::new(100, 100, 400, 300, "Hello from rust");
	let mut frame = Frame::default_fill();
	let mut page = group::Flex::default_fill().column();

	// Set up layout:
	let mut menubar = menu::MenuBar::new(0, 0, 400, 40, "rew");

	let row = group::Flex::default_fill().row();
	let mut left = Rc::from(RefCell::from(PointEditor::new()));
	let mut right = Rc::from(RefCell::from(PointEditor::new()));
	let mut out = Rc::from(RefCell::from(PointEditor::new()));
	row.end();

	let mut but = Button::new(160, 210, 80, 40, "Click me!");

	// Set up callbacks:
	menubar.add("File/New\t", Shortcut::None, menu::MenuFlag::Normal, menu_cb);
	{
		let mut left = left.clone();
		menubar.add("Morph/Open Left Frame\t", Shortcut::None, menu::MenuFlag::Normal, move |m| {
			left.borrow_mut().load_frame_source();
		});
	}
	{
		let mut right = right.clone();
		menubar.add("Morph/Open Right Frame\t", Shortcut::None, menu::MenuFlag::Normal, move |m| {
			right.borrow_mut().load_frame_source();
		});
	}

	// Finalize layout and show:
	page.fixed(&menubar, MENUBAR_HEIGHT);
	page.end();

	wind.make_resizable(true);
	wind.end();
	wind.show();

	//but.set_callback(move |_| frame.set_label("Hello World!")); // the closure capture is mutable borrow to our button
	app.run().unwrap();
	// app.run().unwrap() lets the thread loop run and the app do its thing.
	// while app.wait() {} gives us control over handling messages.
}


fn rs_image_to_fl_image(img: &imagers::DynamicImage) -> Result<imagefl::RgbImage, FltkError> {
	let width = img.width() as usize;
	let height = img.height() as usize;
	let mut converted_image: imagers::RgbImage;
	let mut data: &[u8];
	if let Some(converted) = img.as_rgb8() {
		data = converted.as_raw();
	} else {
		converted_image = img.clone().into_rgb8();
		data = converted_image.as_raw();
	};
	imagefl::RgbImage::new(
		data,
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