//! Multi-viewport docking helpers (inspired by [egui_docking](https://github.com/Latias94/egui_docking)).

mod geometry;
mod ghost;

pub use geometry::{leaf_rect_to_screen, pointer_latest_in_screen};

pub use ghost::show_ghost_preview;

/// Options for [`crate::DockArea::multi_viewport`] behavior.
#[derive(Clone, Debug, PartialEq)]
pub struct MultiViewportOptions {
    /// Show a floating preview while dragging outside all dock areas (before release).
    pub ghost_preview: bool,
    /// Detach into a new native window as soon as the pointer leaves all dock areas mid-drag.
    pub live_tear_off: bool,
    /// Holding ALT while releasing forces tear-off even when the pointer is over a dock.
    pub detach_on_alt: bool,
    /// Holding CTRL while tearing off creates an embedded [`egui::Window`] instead of a native viewport.
    pub contained_window_on_ctrl: bool,
    /// When `true`, holding SHIFT disables docking targets while dragging (ImGui `ConfigDockingWithShift=false`).
    pub disable_docking_while_shift: bool,
}

impl Default for MultiViewportOptions {
    fn default() -> Self {
        Self {
            ghost_preview: true,
            live_tear_off: false,
            detach_on_alt: true,
            contained_window_on_ctrl: true,
            disable_docking_while_shift: false,
        }
    }
}
