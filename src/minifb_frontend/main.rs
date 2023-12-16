use std::sync::{Arc, Mutex};
use minifb::{Key, MouseMode, MouseButton, Window, WindowOptions};
use morph_tool::animation_system::Animation;
use morph_tool::image_source::{FrameProvider, NullImageProvider};

const START_WIDTH: usize = 1280;
const START_HEIGHT: usize = 720;


fn main() {
	// App state:
	let mut left_frame_provider: Arc<Mutex<Box<dyn FrameProvider>>> = Arc::new(Mutex::new(Box::new(NullImageProvider::new())));
	let mut right_frame_provider: Arc<Mutex<Box<dyn FrameProvider>>> = Arc::new(Mutex::new(Box::new(NullImageProvider::new())));
	let mut keyframes: Arc<Mutex<Animation>> = Arc::new(Mutex::new(Animation::new()));

	// GUI state:
	let mut buffer: Vec<u32> = vec![0; START_WIDTH * START_HEIGHT];

	let mut width = START_WIDTH;
	let mut height = START_HEIGHT;
	let mut window = Window::new(
		"Test - ESC to exit",
		START_WIDTH,
		START_HEIGHT,
		WindowOptions {
			resize: true,
			..WindowOptions::default()
		},
	)
		.unwrap_or_else(|e| {
			panic!("{}", e);
		});

	// Limit to max ~60 fps update rate
	window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

	while window.is_open() && !window.is_key_down(Key::Escape) {
		// Draw to buffer:
		/*
		for i in buffer.iter_mut() {
			*i = 0; // write something more funny here!
		}
		*/

		// Handle resize:
		let (new_width, new_height) = window.get_size();
		if new_width != width || new_height != height {
			let mut new_buffer = vec![0; new_width * new_height];
			for y in 0..height.min(new_height) {
				for x in 0..width.min(new_width) {
					new_buffer[y * new_width + x] = buffer[y * width + x];
				}
			}
			buffer = new_buffer;
			width = new_width;
			height = new_height;
		}

		if let Some((x, y)) = window.get_mouse_pos(MouseMode::Discard) {
			let screen_pos = (y as usize) * width + (x as usize);

			if window.get_mouse_down(MouseButton::Left) {
				buffer[screen_pos] = 0x00ffffff;
			}

			if window.get_mouse_down(MouseButton::Right) {
				buffer[screen_pos] = 0;
			}
		}

		if let Some(scroll) = window.get_scroll_wheel() {
			println!("Scrolling {} - {}", scroll.0, scroll.1);
		}

		// We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
		window
			.update_with_buffer(&buffer, width, height)
			.unwrap();
	}
}

trait Widget {
	fn draw(&self, framebuffer: &mut Vec<u32>, buffer_size: (usize, usize), output_rect: (usize, usize, usize, usize));
	fn update(&mut self, window: &mut Window);
}

// Displays a bunch of (f32, f32) points on top of an image.
struct PointPlotter {
	frame_provider: Arc<Mutex<Box<dyn FrameProvider>>>,
	cached_image: Vec<u32>,
	cached_points: Vec<f32>,
	dirty: bool,
	// Need a flag for dirty?  We should have the size of the output rect just in case the cached image changes.

	mouse_down: Option<(f32, f32)>,

	created_point: Option<(f32, f32)>,
	moved_point: Option<(usize, f32, f32)>,
	deleted_point: Option<(f32, f32)>,

	camera_offset: (f32, f32),
	zoom: f32,
}

impl Widget for PointPlotter {
	fn draw(&self, framebuffer: &mut Vec<u32>, buffer_size: (usize, usize), output_rect: (usize, usize, usize, usize)) {
		// We have a rectangle given as (left, top, right, bottom)
		// We could draw outside of that, but...

		// First, draw the cached image.  If the size is different, we can draw just what we have.
		let (left, top, right, bottom) = output_rect;
		let width = right-left;
		let height = bottom-top;

		if width*height != self.cached_image.len() {
			// Need redraw.
			return;
		}

		for buffer_y in top..bottom {
			for buffer_x in left..right {
				let image_x = buffer_x - left;
				let image_y = buffer_y - top;
				if image_x < buffer_size.0 && image_y < buffer_size.1 {
					framebuffer[buffer_x + buffer_y*buffer_size.0] = self.cached_image[image_x + image_y*width];
				}
			}
		}
	}

	fn update(&mut self, window: &mut Window) {
		if let Some((mouse_x, mouse_y)) = window.get_mouse_pos(MouseMode::Pass) {
			// Drag:
			if window.get_mouse_down(MouseButton::Middle) {
				if let Some((prev_x, prev_y)) = self.mouse_down {
					let dx = mouse_x - prev_x;
					let dy = mouse_y - prev_y;
					// We have to set prev because we're accumulating the camera offset and adding it.
					self.mouse_down.replace((mouse_x, mouse_y));
					self.camera_offset = (self.camera_offset.0 + dx, self.camera_offset.1 + dy);
				} else {
					// Start drag.
					self.mouse_down = Some((mouse_x, mouse_y));
				}
			} else {
				// Clear middle drag.
				self.mouse_down = None;
			}
		}
	}
}