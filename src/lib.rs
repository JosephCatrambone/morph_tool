use crate::animation_system::Animation;
use crate::image_source::FrameProvider;
use eframe;
use egui;
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};

pub mod animation_system;
pub mod image_source;
pub mod thin_plate_spline;


/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct MorphApp {
	#[serde(skip)]
	animation: Animation,
	#[serde(skip)]
	left: Box<dyn FrameProvider>,
	#[serde(skip)]
	right: Box<dyn FrameProvider>,
	
	label: String,
	
	#[serde(skip)]
	value: f32,
	
	#[serde(skip)]
	tools: DockState<ToolPane>,
}

enum ToolPane {
	SourcePointEditor,
	DestinationPointEditor,
	TimelineEditor,
	OutputViewer,
}

struct ToolRenderer;

impl TabViewer for ToolRenderer {
	type Tab = ToolPane;
	
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
		match tab {
			ToolPane::SourcePointEditor => "Source Point Editor",
			ToolPane::DestinationPointEditor => "Destination Point Editor",
			ToolPane::TimelineEditor => "Timeline Editor",
			ToolPane::OutputViewer => "Output Viewer",
		}.into()
    }

    // Defines the contents of a given `tab`.
    fn ui(&mut self, ui: &mut egui::Ui, tool_pane: &mut Self::Tab) {
        ui.label(format!("Content of tool pane!"));
    }
}


impl Default for MorphApp {
	fn default() -> Self {
		Self {
			animation: Animation::new(),
			left: Box::new(image_source::NullImageProvider::new()),
			right: Box::new(image_source::NullImageProvider::new()),
			tools: DockState::new(vec![ToolPane::SourcePointEditor, ToolPane::DestinationPointEditor, ToolPane::OutputViewer, ToolPane::TimelineEditor]),
			// Example stuff:
			label: "Hello World!".to_owned(),
			value: 2.7,
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
		
		egui::CentralPanel::default().show(ctx, |ui| {
			DockArea::new(&mut self.tools)
				.style(Style::from_egui(ui.style().as_ref()))
				.show_inside(ui, &mut ToolRenderer);
			ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
				egui::warn_if_debug_build(ui);
			});
		});
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
	}
}
