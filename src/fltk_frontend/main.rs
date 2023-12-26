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
	valuator,
	window::Window,
};
use image as imagers;
use imageproc;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::DerefMut;
use std::sync::{Mutex, Arc};
use fltk::app::MouseButton;
use fltk::draw::draw_circle_fill;
use fltk::enums::Event;
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
const DISTANCE_SELECT_THRESHOLD: f32 = 100f32;

#[derive(Clone, Copy, Debug)]
enum PointEvent {
	PointSelected(usize),
	PointDeselected,
	PointAdded(f32, f32),
	PointMoved(usize, f32, f32), // idx, final_x, final_y
	PointDeleted(usize),
}

struct PointEditor {
	cached_image: Arc<Mutex<imagefl::RgbImage>>,
	cached_keypoints: Arc<Mutex<Vec<f32>>>,
	selected_point: Option<usize>,
	pub editable: bool,
	// Shared:
	pub image_source: Arc<Mutex<Box<dyn FrameProvider>>>,
	// Callbacks:
	pub event_listeners: Arc<Mutex<Vec<Box<dyn Fn(PointEvent) -> ()>>>>,
}

impl PointEditor {
	pub fn new() -> Self {
		let raw_provider = NullImageProvider::new(None);
		let image_source: Arc<Mutex<Box<dyn FrameProvider>>> = Arc::new(Mutex::new(Box::new(raw_provider)));
		let cached_image = Arc::new(Mutex::new(rs_image_to_fl_image(image_source.lock().unwrap().get_frame(0)).unwrap()));
		let keypoints = Arc::new(Mutex::new(Vec::<f32>::new()));
		let callbacks = Arc::new(Mutex::new(vec![]));
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
				points.lock().unwrap().chunks_exact(2).enumerate().for_each(|(idx, point)| {
					match point {
						&[x, y] => draw_circle_fill(x as i32, y as i32, 3, Color::from_rgb(255, 0, 255)),
						_ => println!("Got odd length match."),
					}
				});

			}
		});

		frame.handle({
			let mut listeners = callbacks.clone();
			let mut prev_mouse: Option<(i32, i32)> = None;
			let mut selected_point: Option<usize> = None;
			let mut keypoints = keypoints.clone();
			move |f, evt| {
				let mut result_event: Option<PointEvent> = None;
				let handled = match evt {
					Event::Push => {
						//let (mouse_x, mouse_y) = app::get_mouse();
						let (mouse_x, mouse_y) = app::event_coords();
						match app::event_mouse_button() {
							MouseButton::Left => {
								// Find out if we're selecting or adding.
								let mut nearest_idx = 0usize;
								let mut nearest_distance = 1e10f32;
								let kp = keypoints.lock().unwrap();
								for idx in 0..kp.len()/2 {
									let dx = kp[idx*2] - mouse_x as f32;
									let dy = kp[idx*2 + 1] - mouse_y as f32;
									let dist = dx*dx+dy*dy;
									if dist < nearest_distance {
										nearest_distance = dist;
										nearest_idx = idx;
									}
								}
								if nearest_distance < DISTANCE_SELECT_THRESHOLD {
									selected_point = Some(nearest_idx);
									result_event = Some(PointEvent::PointSelected(nearest_idx));
								} else {
									selected_point = None;
									result_event = Some(PointEvent::PointDeselected);
								}
							},
							MouseButton::Middle => {
							},
							MouseButton::Right => {
								result_event = Some(PointEvent::PointAdded(mouse_x as f32, mouse_y as f32));
							}
							_ => {},
						}
						true
					},
					Event::Drag => {
						let (mouse_x, mouse_y) = app::event_coords();
						let (mouse_dx, mouse_dy) = if let Some((prev_x, prev_y)) = prev_mouse {
							(mouse_x - prev_x, mouse_y - prev_y)
						} else {
							prev_mouse = Some((mouse_x, mouse_y));
							(0, 0)
						};
						match app::event_mouse_button() {
							MouseButton::Left => {
								if let Some(selected_idx) = selected_point {
									let mut kp = keypoints.lock().unwrap();
									kp[selected_idx*2] += mouse_dx as f32;
									kp[selected_idx*2 +1] += mouse_dy as f32;
									result_event = Some(PointEvent::PointMoved(selected_idx, kp[selected_idx*2], kp[selected_idx*2+1]));
								}
							},
							MouseButton::Middle => {},
							_ => {},
						}
						true
					},
					Event::Released => {
						prev_mouse = None;
						true
					}
					_ => false // Event unhandled
				};
				if let Some(evt) = result_event {
					let listeners = listeners.lock().unwrap();
					listeners.iter().for_each(move |f: &Box<dyn Fn(PointEvent) -> ()>|{ f(evt); });
				}
				handled
			}
		});

		Self {
			cached_image,
			image_source,
			editable: true,
			selected_point: None,
			cached_keypoints: keypoints,
			event_listeners: callbacks,
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
	
	fn set_image(&mut self, new_image: &imagers::DynamicImage) {
		let mut img = self.cached_image.lock().unwrap();
		*img = rs_image_to_fl_image(new_image).unwrap();
	}
	
	fn add_point(&mut self, x: f32, y: f32) {
		let mut pts = self.cached_keypoints.lock().unwrap();
		pts.push(x);
		pts.push(y);
	}

	fn set_points(&mut self, new_points: &Vec<f32>) {
		let mut pts = self.cached_keypoints.lock().unwrap();
		pts.clear();
		pts.extend_from_slice(new_points.as_slice());
	}
	
	fn set_selected_point(&mut self, point: Option<usize>) {
		self.selected_point = point;
	}

	fn add_listener(&mut self, f: Box<dyn Fn(PointEvent) -> ()>) {
		self.event_listeners.lock().unwrap().push(f);
	}
}

fn main() {
	let mut animation = Arc::new(Mutex::new(Animation::new()));
	let app = app::App::default();
	let mut wind = Window::new(100, 100, 400, 300, "Hello from rust");
	//let mut frame = Frame::default_fill();
	let mut page = group::Flex::default_fill().column();

	// Set up layout:
	let mut menubar = menu::MenuBar::new(0, 0, 400, 40, "rew");

	let row = group::Flex::default_fill().row();
	let mut left = Rc::from(RefCell::from(PointEditor::new()));
	let mut right = Rc::from(RefCell::from(PointEditor::new()));
	let mut out = Rc::from(RefCell::from(PointEditor::new()));
	row.end();
	
	let row = group::Flex::default_fill().row();
	let mut slider = valuator::Slider::default();
	slider.set_type(valuator::SliderType::HorizontalNice);
	slider.set_bounds(0.0, 1.0);
	slider.set_range(0.0, 1.0);
	slider.set_value(0.5f64);
	slider.set_step(0.0, 1);
	{
		let left = left.clone();
		let right = right.clone();
		let out = out.clone();
		let anim = animation.clone();
		slider.set_callback(move |s| {
			println!("Slider value: {}", s.value());
		});
	}
	row.end();

	//let mut but = Button::new(160, 210, 80, 40, "Click me!");

	// Set up callbacks:
	menubar.add("File/New\t", Shortcut::None, menu::MenuFlag::Normal, menu_cb);
	{
		let mut left = left.clone();
		menubar.add("Morph/Open Left Frame\t", Shortcut::None, menu::MenuFlag::Normal, move |m| {
			left.borrow_mut().load_frame_source();
			app::redraw();
		});
	}
	{
		let mut right = right.clone();
		menubar.add("Morph/Open Right Frame\t", Shortcut::None, menu::MenuFlag::Normal, move |m| {
			right.borrow_mut().load_frame_source();
			app::redraw();
		});
	}
	let generate_and_add_callbacks = move |origin: Rc<RefCell<PointEditor>>, dest: Rc<RefCell<PointEditor>>| {
		let o2 = origin.clone();
		let d2 = dest.clone();
		let animation = animation.clone();
		let f = Box::new(move |evt: PointEvent| {
			match evt {
				PointEvent::PointAdded(x, y) => {
					animation.lock().unwrap().set_point(x, y, x, y, 0, None);
					// Add this point at both the origin and destination.
					o2.borrow_mut().add_point(x, y);
					d2.borrow_mut().add_point(x, y);
					app::redraw();
				},
				PointEvent::PointSelected(idx) => {
					o2.borrow_mut().selected_point = Some(idx);
					d2.borrow_mut().selected_point = Some(idx);
				},
				PointEvent::PointDeselected => {
					o2.borrow_mut().selected_point = None;
					d2.borrow_mut().selected_point = None;
				},
				PointEvent::PointMoved(idx, x, y) => {
					// TODO: Start here.
					// Change set_point to take None.
					//animation.lock().unwrap().
				},
				_ => println!("asdf"),
			}
		});
		origin.borrow_mut().add_listener(f);
	};
	generate_and_add_callbacks(left.clone(), right.clone());
	generate_and_add_callbacks(right.clone(), left.clone());

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