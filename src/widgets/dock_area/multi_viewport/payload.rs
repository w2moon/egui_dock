use std::sync::Arc;

use egui::{Context, ViewportId};

use crate::{SurfaceIndex, TabPath};

/// Cross-viewport drag payload (egui [`DragAndDrop`](egui::DragAndDrop)).
#[derive(Clone, Debug)]
pub struct DockDragPayload {
    /// [`DockArea`](crate::DockArea) that owns the dragged tab.
    pub dock_area_id: egui::Id,
    /// Viewport where the drag started.
    pub source_viewport: ViewportId,
    /// Surface index at drag start.
    pub source_surface: SurfaceIndex,
    /// Tab being dragged.
    pub tab_path: TabPath,
}

impl DockDragPayload {
    /// Store payload for other viewports to read on drop.
    pub fn set(ctx: &Context, payload: Self) {
        egui::DragAndDrop::set_payload(ctx, payload);
    }

    /// Peek at the active payload without consuming it.
    pub fn get(ctx: &Context) -> Option<Arc<Self>> {
        egui::DragAndDrop::payload(ctx)
    }

    /// Take ownership of the payload (typically on drop).
    pub fn take(ctx: &Context) -> Option<Arc<Self>> {
        egui::DragAndDrop::take_payload(ctx)
    }

    /// Clear any in-flight payload.
    pub fn clear(ctx: &Context) {
        egui::DragAndDrop::clear_payload(ctx);
    }
}
