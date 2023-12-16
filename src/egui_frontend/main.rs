#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use egui::{Pos2, Vec2, Rect, Ui, WidgetText};
use egui_extras;
use image::DynamicImage;
use eframe;
use egui;
use env_logger;
use epaint::TextureId;
use morph_tool::*;
use morph_tool::animation_system::Animation;
use morph_tool::image_source::{FrameProvider, open_image_source};

fn main() -> eframe::Result<()> {
	env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

	let native_options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default()
			.with_inner_size([400.0, 300.0])
			.with_min_inner_size([300.0, 220.0]),
		..Default::default()
	};
	eframe::run_native(
		"Morph Tool",
		native_options,
		Box::new(|cc| Box::new(MorphApp::new(cc))),
	)
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
struct MorphApp {
	#[serde(skip)]
	animation: Animation,
	#[serde(skip)]
	left: Box<dyn FrameProvider>,
	#[serde(skip)]
	right: Box<dyn FrameProvider>,

	#[serde(skip)]
	cached_frame_left: Option<egui::Image>,
	#[serde(skip)]
	cached_points_left: Vec<f32>,
	#[serde(skip)]
	cached_frame_right: Option<TextureId>,
	#[serde(skip)]
	cached_points_right: Vec<f32>,

	label: String,
	current_frame: u32,
}

impl Default for MorphApp {
	fn default() -> Self {
		Self {
			animation: Animation::new(),
			left: Box::new(image_source::NullImageProvider::new()),
			right: Box::new(image_source::NullImageProvider::new()),

			cached_points_left: vec![],
			cached_points_right: vec![],
			cached_frame_left: None,
			cached_frame_right: None,

			label: "Hello World!".to_owned(),
			current_frame: 0u32,
		}
	}
}

impl MorphApp {
	/// Called once before the first frame.
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		// This is also where you can customize the look and feel of egui using
		// `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

		egui_extras::install_image_loaders(&cc.egui_ctx);

		// Load previous app state (if any).
		// Note that you must enable the `persistence` feature for this to work.
		if let Some(storage) = cc.storage {
			return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
		}

		Default::default()
	}
}

impl eframe::App for MorphApp {
	/// Called by the frame work to save state before shutdown.
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		eframe::set_value(storage, eframe::APP_KEY, self);
	}

	/// Called each time the UI needs repainting, which may be many times per second.
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
		// For inspiration and more examples, go to https://emilk.github.io/egui

		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			// The top panel is often a good place for a menu bar:
			egui::menu::bar(ui, |ui| {
				//let is_web = cfg!(target_arch = "wasm32");
				ui.menu_button("File", |ui| {
					if ui.buttom("Load Left Image").clicked() {
						if let Some(img_source) = open_image_source() {
							self.left = img_source;
							let img = self.left.get_frame(0);
						}
					}
					if ui.button("Quit").clicked() {
						ctx.send_viewport_cmd(egui::ViewportCommand::Close);
					}
				});
				ui.add_space(16.0);

				egui::widgets::global_dark_light_mode_buttons(ui);
			});
		});

		egui::SidePanel::left("left_panel").show(ctx, |ui|{

		});
		egui::CentralPanel::default().show(ctx, |ui| {

		});
		egui::SidePanel::right("right_panel").show(ctx, |ui|{

		});
	}
}

fn point_panel(ctx: &egui::Context, ui: &mut Ui, img_id: TextureId, img: &DynamicImage, pts: &mut Vec<f32>, camera_offset_x: &mut f32, camera_offset_y: &mut f32, camera_zoom: &mut f32) -> egui::Response {
	let (mut response, painter) = ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::drag());

	let to_screen = egui::emath::RectTransform::from_to(
		Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
		response.rect,
	);
	let from_screen = to_screen.inverse();

	// Draw image with zoom and offset.
	let img_rect = Rect::from_center_size(Pos2::new(*camera_offset_x, *camera_offset_y), Vec2::new(img.width()*camera_zoom, img.height()*camera_zoom));
	egui::Image::new(img_id).paint_at(ui, img_rect);

	if let Some(pointer_pos) = response.interact_pointer_pos() {
		let canvas_pos = from_screen * pointer_pos;
		if current_line.last() != Some(&canvas_pos) {
			current_line.push(canvas_pos);
			response.mark_changed();
		}
	} else if !current_line.is_empty() {
		self.lines.push(vec![]);
		response.mark_changed();
	}

	let shapes = self
		.lines
		.iter()
		.filter(|line| line.len() >= 2)
		.map(|line| {
			let points: Vec<Pos2> = line.iter().map(|p| to_screen * *p).collect();
			egui::Shape::line(points, self.stroke)
		});

	painter.extend(shapes);

	response
}

