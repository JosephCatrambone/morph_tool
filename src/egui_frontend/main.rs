#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use egui::{Ui, WidgetText};
use egui_tiles::{TileId, UiResponse};
use image::DynamicImage;
use eframe;
use egui;
use env_logger;
use morph_tool::*;
use morph_tool::animation_system::Animation;
use morph_tool::image_source::FrameProvider;

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
	cached_points_left: Vec<f32>,
	#[serde(skip)]
	cached_points_right: Vec<f32>,

	label: String,
	current_frame: u32,
}

struct TabViewer;

struct TabData {
	pane_title: String,
	frame_number: u32,
	point_data: Vec<f32>,
	image_data: DynamicImage,
}

impl egui_tiles::Behavior<TabData> for TabViewer {
	fn pane_ui(&mut self, ui: &mut Ui, _tile_id: TileId, pane: &mut TabData) -> UiResponse {
		if ui
			.add(egui::Button::new(&pane.pane_title).sense(egui::Sense::drag()))
			.drag_started()
		{
			egui_tiles::UiResponse::DragStarted
		} else {
			egui_tiles::UiResponse::None
		}
	}

	fn tab_title_for_pane(&mut self, pane: &TabData) -> WidgetText {
		pane.pane_title.clone().into()
	}
}

impl Default for MorphApp {
	fn default() -> Self {
		Self {
			animation: Animation::new(),
			left: Box::new(image_source::NullImageProvider::new()),
			right: Box::new(image_source::NullImageProvider::new()),

			cached_points_left: vec![],
			cached_points_right: vec![],

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

