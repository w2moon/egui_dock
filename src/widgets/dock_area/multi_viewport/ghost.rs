use egui::{Area, Context, Frame, Order, Pos2, Vec2, WidgetText};

use super::geometry::pointer_latest_in_screen;
use super::ghost_drag::GhostDragMode;
use crate::{
    dock_area::{drag_and_drop::TreeComponent, state::State},
    DockArea, TabDestination, TabInsert, TabViewer,
};

/// Floating preview of the dragged tab while the pointer is outside all dock rects.
pub fn show_ghost_preview<Tab>(
    dock_area: &DockArea<'_, Tab>,
    ctx: &Context,
    state: &State,
    title: WidgetText,
) {
    if dock_area.is_contained_ghost_active(state) {
        return;
    }

    let Some(pointer_screen) = state
        .mv_drag
        .pointer_global_fallback(ctx)
        .or_else(|| pointer_latest_in_screen(ctx))
    else {
        return;
    };

    if dock_area.pointer_over_dock_for_ghost(ctx, state, pointer_screen) {
        return;
    }

    let grab = Vec2::new(48.0, 12.0);
    let pos = pointer_screen - grab;

    Area::new(dock_area.id.with("multi_viewport_ghost"))
        .order(Order::Tooltip)
        .current_pos(pos)
        .show(ctx, |ui| {
            Frame::window(ui.style())
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.label(title);
                });
        });
}

impl<Tab> DockArea<'_, Tab> {
    /// Dock hit test for ghost tear-off / preview, excluding the ghost's own floating panel.
    pub(super) fn pointer_over_dock_for_ghost(
        &self,
        ctx: &Context,
        state: &State,
        pointer_screen: Pos2,
    ) -> bool {
        self.pointer_over_dock_screen(ctx, state, pointer_screen, 2.0, true)
    }

    pub(in crate::widgets::dock_area) fn try_live_tear_off(
        &mut self,
        ctx: &Context,
        state: &mut State,
        tab_viewer: &impl TabViewer<Tab = Tab>,
    ) {
        if state.ghost_drag.is_some() || state.live_tear_off_surface.is_some() {
            return;
        }

        let Some(dnd) = state.dnd.as_ref() else {
            return;
        };

        let TreeComponent::Tab(src) = dnd.drag.src else {
            return;
        };

        let Some(pointer_screen) = state
            .mv_drag
            .pointer_global_fallback(ctx)
            .or_else(|| pointer_latest_in_screen(ctx))
        else {
            return;
        };

        if self.pointer_over_dock_screen(ctx, state, pointer_screen, 0.0, true) {
            return;
        }

        let allowed_in_windows = {
            let Some(leaf) = self.dock_state[src.node_path()].get_leaf_mut() else {
                return;
            };
            tab_viewer.allowed_in_windows(&mut leaf.tabs[src.tab.0])
        };
        if !allowed_in_windows {
            return;
        }

        let size = dnd.drag.rect.size();
        let ctrl = ctx.input(|i| i.modifiers.ctrl);
        if ctrl && self.multi_viewport_options.tear_off_to_floating_on_ctrl {
            self.tear_off_to_floating_panel(ctx, src);
            return;
        }

        let use_native = self.multi_viewport
            && !(ctrl && self.multi_viewport_options.contained_window_on_ctrl);

        let window_rect = egui::Rect::from_min_size(pointer_screen, size);
        let new_surface = if use_native {
            self.dock_state.detach_tab(src, window_rect)
        } else {
            let idx = self.dock_state.detach_tab(src, window_rect);
            if let Some(ws) = self.dock_state.get_window_state_mut(idx) {
                ws.set_native_viewport(false);
            }
            idx
        };

        state.live_tear_off_surface = Some(new_surface);

        let new_path = crate::TabPath {
            surface: new_surface,
            node: crate::NodeIndex::root(),
            tab: crate::TabIndex(0),
        };
        if let Some(dnd_mut) = state.dnd.as_mut() {
            dnd_mut.drag.src = TreeComponent::Tab(new_path);
        }
        ctx.request_repaint();
    }

    /// ImGui-style ghost tear-off: extract the tab while dragging, ESC restores, release docks.
    pub(in crate::widgets::dock_area) fn try_ghost_tear_off(
        &mut self,
        ctx: &Context,
        state: &mut State,
        tab_viewer: &impl TabViewer<Tab = Tab>,
    ) {
        if state.ghost_drag.is_some() || state.live_tear_off_surface.is_some() {
            return;
        }

        let Some(dnd) = state.dnd.as_ref() else {
            return;
        };

        let TreeComponent::Tab(src) = dnd.drag.src else {
            return;
        };

        if ctx.input(|i| i.pointer.any_released()) {
            return;
        }

        let Some(pointer_screen) = state
            .mv_drag
            .pointer_global_fallback(ctx)
            .or_else(|| pointer_latest_in_screen(ctx))
        else {
            return;
        };

        let threshold = self.multi_viewport_options.ghost_tear_off_threshold;
        if self.pointer_over_dock_screen(ctx, state, pointer_screen, threshold, true) {
            return;
        }

        let host = ctx.viewport_id();
        if self
            .contained_floating_under_pointer(ctx, host, pointer_screen, None)
            .is_some()
        {
            return;
        }

        let allowed_in_windows = {
            let Some(leaf) = self.dock_state[src.node_path()].get_leaf_mut() else {
                return;
            };
            tab_viewer.allowed_in_windows(&mut leaf.tabs[src.tab.0])
        };
        if !allowed_in_windows {
            return;
        }

        let restore = TabDestination::Node(src.node_path(), TabInsert::Insert(src.tab));
        let size = dnd.drag.rect.size();
        let mode = self.ghost_mode_for_tear_off(ctx);

        if mode == GhostDragMode::ContainedFloating && !self.multi_viewport {
            return;
        }

        self.start_ghost_drag(ctx, state, src, restore, mode, pointer_screen, size);
    }

    fn ghost_mode_for_tear_off(&self, ctx: &Context) -> GhostDragMode {
        let ctrl = ctx.input(|i| i.modifiers.ctrl);
        if ctrl && self.multi_viewport_options.tear_off_to_floating_on_ctrl {
            return GhostDragMode::ContainedFloating;
        }
        if self.multi_viewport && !self.multi_viewport_options.ghost_spawn_native_on_leave_dock {
            return GhostDragMode::ContainedFloating;
        }
        if self.multi_viewport {
            GhostDragMode::Native
        } else {
            GhostDragMode::ContainedFloating
        }
    }

    fn pointer_over_dock_screen(
        &self,
        ctx: &Context,
        state: &State,
        pointer_screen: Pos2,
        threshold: f32,
        exclude_ghost_panel: bool,
    ) -> bool {
        let ghost_surface = exclude_ghost_panel.then(|| self.ghost_torn_surface(state)).flatten();

        let over_leaf = state.dock_rects_screen.iter().any(|hit| {
            if ghost_surface == Some(hit.surface) {
                return false;
            }
            hit.rect.expand(threshold).contains(pointer_screen)
        });

        over_leaf || {
            let main_inner = ctx.input(|i| {
                i.raw
                    .viewports
                    .get(&egui::ViewportId::ROOT)?
                    .inner_rect
            });
            main_inner.is_some_and(|inner| {
                inner.expand(threshold).contains(pointer_screen)
            }) && self.dock_state.main_surface().is_empty()
        }
    }
}
