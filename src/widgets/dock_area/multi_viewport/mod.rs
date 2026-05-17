//! Multi-viewport docking helpers (inspired by [egui_docking](https://github.com/Latias94/egui_docking)).

mod contained_floating;
mod drag_state;
mod drop;
mod floating_z;
mod geometry;
mod ghost;
mod ghost_drag;
mod payload;
mod pending_drop;

pub use drag_state::MultiViewportDragState;
pub use ghost_drag::{GhostDrag, GhostDragMode};
pub use geometry::{leaf_rect_to_screen, pointer_latest_in_screen};
pub use ghost::show_ghost_preview;
pub use payload::DockDragPayload;
pub use pending_drop::PendingDrop;

/// Options for [`crate::DockArea::multi_viewport`] behavior.
#[derive(Clone, Debug, PartialEq)]
pub struct MultiViewportOptions {
    /// Show a floating preview while dragging outside all dock areas (before release).
    pub ghost_preview: bool,
    /// Detach into a new native window as soon as the pointer leaves all dock areas mid-drag.
    pub live_tear_off: bool,
    /// Holding ALT while releasing forces tear-off even when the pointer is over a dock.
    pub detach_on_alt: bool,
    /// Holding CTRL while tearing off spawns a contained floating panel inside the viewport (egui [`Area`]).
    pub tear_off_to_floating_on_ctrl: bool,
    /// Holding CTRL while tearing off creates an embedded [`egui::Window`] instead of a native viewport.
    /// Ignored when [`Self::tear_off_to_floating_on_ctrl`] handles the tear-off first.
    pub contained_window_on_ctrl: bool,
    /// When `true`, holding SHIFT disables docking targets while dragging (ImGui `ConfigDockingWithShift=false`).
    pub disable_docking_while_shift: bool,
    /// Mid-drag tear-off when the pointer leaves all dock rects (ImGui ghost docking). Re-docks on release like normal drag.
    pub ghost_tear_off: bool,
    /// Expand dock hit area before starting ghost tear-off (screen points).
    pub ghost_tear_off_threshold: f32,
    /// When ghost tear-off starts outside the dock, spawn a native viewport immediately.
    /// When `false`, spawns a contained [`egui::Area`] panel in the source viewport instead.
    pub ghost_spawn_native_on_leave_dock: bool,
}

impl Default for MultiViewportOptions {
    fn default() -> Self {
        Self {
            ghost_preview: true,
            live_tear_off: false,
            detach_on_alt: true,
            tear_off_to_floating_on_ctrl: true,
            contained_window_on_ctrl: false,
            disable_docking_while_shift: false,
            ghost_tear_off: false,
            ghost_tear_off_threshold: 0.0,
            ghost_spawn_native_on_leave_dock: true,
        }
    }
}
