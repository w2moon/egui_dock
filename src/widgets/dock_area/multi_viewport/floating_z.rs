use egui::{Pos2, ViewportId};

use crate::{DockArea, SurfaceIndex};

use super::geometry::{leaf_rect_to_screen, viewport_for_surface};

impl<Tab> DockArea<'_, Tab> {
    /// Raises a contained floating surface above others in the same viewport.
    pub(in crate::widgets::dock_area) fn bring_contained_floating_to_front(
        &mut self,
        host_viewport: ViewportId,
        surface: SurfaceIndex,
    ) {
        let mut max_rank = 0u32;
        for &s in self.dock_state.valid_surface_indices().iter() {
            if s.is_main() {
                continue;
            }
            let Some(ws) = self.dock_state.get_window_state(s) else {
                continue;
            };
            if !ws.is_floating_in_viewport() {
                continue;
            }
            if viewport_for_surface(self.id, s) != host_viewport {
                continue;
            }
            max_rank = max_rank.max(ws.floating_z_rank());
        }
        if let Some(ws) = self.dock_state.get_window_state_mut(surface) {
            ws.set_floating_z_rank(max_rank.saturating_add(1));
        }
    }

    pub(in crate::widgets::dock_area) fn register_contained_floating(
        &mut self,
        host_viewport: ViewportId,
        surface: SurfaceIndex,
    ) {
        self.bring_contained_floating_to_front(host_viewport, surface);
    }

    pub(in crate::widgets::dock_area) fn sort_contained_floating_surfaces(
        &mut self,
        surfaces: &mut [SurfaceIndex],
    ) {
        surfaces.sort_by_key(|s| {
            self.dock_state
                .get_window_state(*s)
                .map(|ws| ws.floating_z_rank())
                .unwrap_or(0)
        });
    }

    /// Topmost contained floating panel under `pointer_screen`, if any.
    pub(in crate::widgets::dock_area) fn contained_floating_under_pointer(
        &mut self,
        ctx: &egui::Context,
        host_viewport: ViewportId,
        pointer_screen: Pos2,
        exclude: Option<SurfaceIndex>,
    ) -> Option<SurfaceIndex> {
        let mut surfaces: Vec<SurfaceIndex> = self
            .dock_state
            .valid_surface_indices()
            .iter()
            .copied()
            .filter(|s| !s.is_main())
            .filter(|&s| {
                self.dock_state
                    .get_window_state(s)
                    .is_some_and(|ws| ws.is_floating_in_viewport())
            })
            .filter(|&s| viewport_for_surface(self.id, s) == host_viewport)
            .collect();

        self.sort_contained_floating_surfaces(&mut surfaces);

        for surface in surfaces.into_iter().rev() {
            if exclude == Some(surface) {
                continue;
            }
            let Some(local_rect) = self.dock_state[surface][crate::NodeIndex::root()].rect() else {
                continue;
            };
            let Some(screen) = leaf_rect_to_screen(ctx, self.id, surface, local_rect) else {
                continue;
            };
            if screen.contains(pointer_screen) {
                return Some(surface);
            }
        }
        None
    }
}
