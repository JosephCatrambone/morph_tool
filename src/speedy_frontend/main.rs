use image::EncodableLayout;
use speedy2d::color::Color;
use speedy2d::{Graphics2D, Window};
use speedy2d::dimen::{Vec2, Vector2};
use speedy2d::image as imagesp;
use speedy2d::window::*;
use morph_tool::animation_system::Animation;
use morph_tool::image_source::*;

fn main() {
	let window = Window::new_centered("Title", (640, 480)).unwrap();
	window.run_loop(MorphTool::new());
}

struct MorphTool {
	// Logic:
	left: Box<dyn FrameProvider>,
	right: Box<dyn FrameProvider>,
	animation: Animation,
	
	current_frame: u32,
	
	// UI:
	left_image_offset: Vec2,
	right_image_offset: Vec2,
	cached_left_image: Option<imagesp::ImageHandle>,
	cached_right_image: Option<imagesp::ImageHandle>,
	last_mouse_pos: Option<Vec2>,
	lmb_down: bool,
	mmb_down: bool,
	rmb_down: bool,
}

impl WindowHandler for MorphTool
{
	fn on_start(&mut self, helper: &mut WindowHelper<()>, info: WindowStartupInfo) {
	}
	fn on_draw(&mut self, helper: &mut WindowHelper, graphics: &mut Graphics2D) {
		// We have to do loading here because it's the only place with graphics context.
		if self.cached_left_image.is_none() {
			let left_img = self.left.get_frame(self.current_frame);
			let data = left_img.as_bytes();
			let ref_image = graphics.create_image_from_raw_pixels(imagesp::ImageDataType::RGBA, imagesp::ImageSmoothingMode::Linear, (left_img.width(), left_img.height()), data).unwrap();
			self.cached_left_image = Some(ref_image);
		}
		graphics.clear_screen(Color::from_rgb(0.8, 0.9, 1.0));
		graphics.draw_image(self.left_image_offset, self.cached_left_image.as_ref().unwrap());
		helper.request_redraw();
	}
	
	fn on_mouse_move(&mut self, helper: &mut WindowHelper, position: Vec2) {
		log::info!(
			"Got on_mouse_move callback: ({:.1}, {:.1})",
			position.x,
			position.y
		);

		self.last_mouse_pos = Some(position);

		helper.request_redraw();
	}

	fn on_mouse_button_down(&mut self, helper: &mut WindowHelper, button: MouseButton) {
		log::info!("Got on_mouse_button_down callback: {:?}", button);

		if button == MouseButton::Left {
			self.lmb_down = true;
		}

		helper.request_redraw();
	}

	fn on_mouse_button_up(&mut self, helper: &mut WindowHelper, button: MouseButton) {
		log::info!("Got on_mouse_button_up callback: {:?}", button);

		if button == MouseButton::Left {
			self.lmb_down = false;
		}

		helper.request_redraw();
	}

	fn on_mouse_wheel_scroll(
		&mut self,
		_helper: &mut WindowHelper<()>,
		delta: MouseScrollDistance
	) {
		log::info!("Got on_mouse_wheel_scroll callback: {:?}", delta);
	}

	fn on_key_down(
		&mut self,
		_helper: &mut WindowHelper,
		virtual_key_code: Option<VirtualKeyCode>,
		scancode: KeyScancode
	) {
		log::info!(
			"Got on_key_down callback: {:?}, scancode {}",
			virtual_key_code,
			scancode
		);
	}

	fn on_key_up(
		&mut self,
		_helper: &mut WindowHelper,
		virtual_key_code: Option<VirtualKeyCode>,
		scancode: KeyScancode
	) {
		log::info!(
			"Got on_key_up callback: {:?}, scancode {}",
			virtual_key_code,
			scancode
		);
	}

	fn on_keyboard_char(&mut self, _helper: &mut WindowHelper, unicode_codepoint: char) {
		log::info!("Got on_keyboard_char callback: '{}'", unicode_codepoint);
	}

	fn on_keyboard_modifiers_changed(
		&mut self,
		_helper: &mut WindowHelper,
		state: ModifiersState
	) {
		log::info!("Got on_keyboard_modifiers_changed callback: {:?}", state);
	}
}

impl MorphTool {
	pub fn new() -> Self {
		Self {
			left: Box::new(NullImageProvider::new(None)),
			right: Box::new(NullImageProvider::new(None)),
			animation: Animation::new(),
			
			current_frame: 0,
			
			left_image_offset: Vector2::ZERO,
			right_image_offset: Vector2::ZERO,
			cached_left_image: None,
			cached_right_image: None,
			last_mouse_pos: None,
			lmb_down: false,
			mmb_down: false,
			rmb_down: false,
		}
	}
}