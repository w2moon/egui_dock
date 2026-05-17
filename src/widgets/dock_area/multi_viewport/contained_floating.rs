use egui::{Area, Frame, Order, Pos2, Ui, ViewportId};

use crate::{
    dock_area::state::State,
    DockArea, SurfaceIndex, TabViewer,
};

use super::geometry::{leaf_rect_to_screen, viewport_for_surface};

impl<Tab> DockArea<'_, Tab> {
    pub(in crate::widgets::dock_area) fn show_contained_floating_surfaces(
        &mut self,
        ui: &mut Ui,
        host_viewport: ViewportId,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        state: &mut State,
    ) {
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

        let inner_min = ui.ctx().input(|i| {
            i.raw
                .viewports
                .get(&host_viewport)
                .and_then(|v| v.inner_rect)
                .map(|r| r.min)
        });

        let ghost_surface = self.ghost_torn_surface(state);
        let ghost_active_drag = state.dnd.is_some();

        for surf_index in surfaces {
            let is_ghost_panel =
                ghost_surface == Some(surf_index) && self.is_contained_ghost_active(state);

            let (pos, size) = {
                let ws = self.dock_state.get_window_state(surf_index).unwrap();
                (
                    ws.viewport_local_position_or(Pos2::new(32.0, 32.0)),
                    ws.floating_size_hint(),
                )
            };

            let area_pos = inner_min.map(|m| m + pos.to_vec2());

            let order = if is_ghost_panel {
                Order::Tooltip
            } else {
                Order::Foreground
            };

            let area_id = if is_ghost_panel {
                self.id.with(("contained_float_ghost", surf_index.0))
            } else {
                self.id.with(("contained_float", surf_index.0))
            };

            let response = Area::new(area_id)
                .order(order)
                .current_pos(area_pos.unwrap_or(pos))
                .show(ui.ctx(), |ui| {
                    let frame = if is_ghost_panel {
                        Frame::window(ui.style()).stroke(egui::Stroke::new(
                            1.5,
                            ui.visuals().selection.stroke.color,
                        ))
                    } else {
                        Frame::window(ui.style())
                    };
                    frame.show(ui, |ui| {
                        ui.set_min_size(size);
                        if is_ghost_panel {
                            ui.label(egui::RichText::new("Ghost").small().weak());
                            ui.separator();
                        }
                        self.render_nodes(ui, tab_viewer, state, surf_index, None);
                    });
                    ui.response()
                });

            if response.response.clicked() || response.response.drag_started() {
                self.bring_contained_floating_to_front(host_viewport, surf_index);
            }

            let allow_panel_drag = !(is_ghost_panel && ghost_active_drag);
            if allow_panel_drag && response.response.dragged() {
                if let (Some(inner), Some(pointer)) = (inner_min, ui.ctx().pointer_latest_pos()) {
                    let local = pointer - inner.to_vec2();
                    if let Some(ws) = self.dock_state.get_window_state_mut(surf_index) {
                        ws.set_viewport_local_position_persist(local);
                    }
                }
            }

            if let Some(node_rect) = self.dock_state[surf_index][crate::NodeIndex::root()].rect() {
                if let Some(screen) =
                    leaf_rect_to_screen(ui.ctx(), self.id, surf_index, node_rect)
                {
                    state.push_dock_rect_screen(surf_index, crate::NodeIndex::root(), screen);
                }
            }
        }
    }

}
