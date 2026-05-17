use egui::{Context, Pos2, Rect, ViewportId};

use crate::{SurfaceIndex, WindowState};

/// Pointer in screen space for the active viewport (`inner_rect.min + local`).
pub fn pointer_pos_in_global(ctx: &Context) -> Option<Pos2> {
    ctx.input(|i| {
        let pos = i.pointer.latest_pos()?;
        let inner = i.viewport().inner_rect?;
        Some(inner.min + pos.to_vec2())
    })
}

/// Viewport whose `inner_rect` contains `pointer_global` (topmost wins).
pub fn viewport_under_pointer_global(ctx: &Context, pointer_global: Pos2) -> Option<ViewportId> {
    ctx.input(|i| {
        let mut best: Option<(ViewportId, f32)> = None;
        for (id, vp) in &i.raw.viewports {
            let Some(inner) = vp.inner_rect else {
                continue;
            };
            if !inner.contains(pointer_global) {
                continue;
            }
            let area = inner.width() * inner.height();
            match best {
                None => best = Some((*id, area)),
                Some((_, best_area)) if area < best_area => best = Some((*id, area)),
                _ => {}
            }
        }
        best.map(|(id, _)| id)
    })
}

/// Viewport hosting a given dock surface.
#[inline]
pub fn viewport_for_surface(dock_area_id: egui::Id, surface: SurfaceIndex) -> ViewportId {
    if surface.is_main() {
        ViewportId::ROOT
    } else {
        WindowState::viewport_id(dock_area_id, surface)
    }
}

/// Converts a leaf rect from viewport-local space to screen space.
pub fn leaf_rect_to_screen(
    ctx: &Context,
    dock_area_id: egui::Id,
    surface: SurfaceIndex,
    local_rect: Rect,
) -> Option<Rect> {
    let viewport_id = viewport_for_surface(dock_area_id, surface);
    let inner_min = ctx.input(|i| {
        i.raw
            .viewports
            .get(&viewport_id)?
            .inner_rect
            .map(|r| r.min)
    })?;
    Some(local_rect.translate(inner_min.to_vec2()))
}

/// Latest pointer position in screen space for the active viewport.
pub fn pointer_latest_in_screen(ctx: &Context) -> Option<Pos2> {
    pointer_pos_in_global(ctx)
}
