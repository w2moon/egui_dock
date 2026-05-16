#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{egui, NativeOptions};
use egui::ViewportBuilder;
use egui_dock::tab_viewer::OnCloseResponse;
use egui_dock::{DockArea, DockState, MultiViewportOptions, Style};

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
            ui.label("拖出停靠区外松开 → 原生 OS 窗口");
            ui.label("拖回任意窗口的停靠区 → 合并");
            ui.separator();
            ui.label("SHIFT：按住时禁用停靠目标（仅拆出新窗口）");
            ui.label("ALT：任意位置松开强制拆出");
            ui.label("CTRL：拆出为当前视口内的浮动面板（可拖动标题栏）");
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
            .multi_viewport_options(MultiViewportOptions {
                ghost_preview: true,
                live_tear_off: false,
                ..MultiViewportOptions::default()
            })
            .show_inside(ui, &mut TabViewer);
    }
}
