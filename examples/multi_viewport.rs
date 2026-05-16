#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{egui, NativeOptions};
use egui::ViewportBuilder;
use egui_dock::tab_viewer::OnCloseResponse;
use egui_dock::{DockArea, DockState, Style};

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([960.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui_dock multi-viewport",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

struct TabViewer;

impl egui_dock::TabViewer for TabViewer {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.as_str().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        ui.vertical_centered(|ui| {
            ui.heading(tab);
            ui.separator();
            ui.label("将标签拖出停靠区边缘可拆分为原生 OS 窗口。");
            ui.label("可将标签拖回主窗口或其他已拆分的窗口进行合并。");
        });
    }

    fn on_close(&mut self, _tab: &mut Self::Tab) -> OnCloseResponse {
        OnCloseResponse::Close
    }
}

struct MyApp {
    dock_state: DockState<String>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            dock_state: DockState::new(vec![
                "Scene".to_owned(),
                "Inspector".to_owned(),
                "Console".to_owned(),
            ]),
        }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        DockArea::new(&mut self.dock_state)
            .style(Style::from_egui(ui.style().as_ref()))
            .multi_viewport(true)
            .show_inside(ui, &mut TabViewer);
    }
}
