use egui::{Context, Key, Pos2, Vec2, ViewportCommand, ViewportId};

use crate::{
    dock_area::{
        drag_and_drop::TreeComponent,
        state::State,
    },
    NodeIndex, SurfaceIndex, TabDestination, TabIndex, TabPath, WindowState,
};

use super::geometry::viewport_for_surface;
use super::payload::DockDragPayload;

/// How a ghost tear-off is presented while dragging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GhostDragMode {
    /// Detached native OS viewport.
    Native,
    /// [`egui::Area`] panel inside [`GhostDrag::host_viewport`].
    ContainedFloating,
}

/// Mid-drag ghost tear-off state (tab extracted until drop, ESC, or finalize).
#[derive(Debug)]
pub struct GhostDrag {
    /// Where to re-insert the tab when the user presses ESC.
    pub restore: TabDestination,
    pub torn_surface: SurfaceIndex,
    pub mode: GhostDragMode,
    /// Viewport that hosts a [`GhostDragMode::ContainedFloating`] panel.
    pub host_viewport: ViewportId,
    pub grab_offset: Vec2,
}

use crate::DockArea;

impl<Tab> DockArea<'_, Tab> {
    /// Begin ghost drag after the tab was extracted from the tree.
    pub(in crate::widgets::dock_area) fn start_ghost_drag(
        &mut self,
        ctx: &Context,
        state: &mut State,
        src: TabPath,
        restore: TabDestination,
        mode: GhostDragMode,
        pointer_screen: Pos2,
        size: Vec2,
    ) {
        let host_viewport = viewport_for_surface(self.id, src.surface);
        let window_rect = egui::Rect::from_min_size(pointer_screen, size);
        let grab_offset = Vec2::new(20.0, 10.0);

        let torn_surface = match mode {
            GhostDragMode::Native => self.dock_state.detach_tab(src, window_rect),
            GhostDragMode::ContainedFloating => {
                let surface = self.dock_state.detach_tab(src, window_rect);
                let local_pos = self.pointer_screen_to_viewport_local(
                    ctx,
                    host_viewport,
                    pointer_screen,
                );
                if let Some(ws) = self.dock_state.get_window_state_mut(surface) {
                    ws.set_floating_in_viewport(true);
                    ws.set_size(size);
                    if let Some(local) = local_pos {
                        ws.set_viewport_local_position(local - grab_offset);
                    }
                }
                self.register_contained_floating(host_viewport, surface);
                surface
            }
        };

        state.ghost_drag = Some(GhostDrag {
            restore,
            torn_surface,
            mode,
            host_viewport,
            grab_offset,
        });
        state.live_tear_off_surface = Some(torn_surface);

        let new_path = TabPath {
            surface: torn_surface,
            node: NodeIndex::root(),
            tab: TabIndex(0),
        };
        if let Some(dnd_mut) = state.dnd.as_mut() {
            dnd_mut.drag.src = TreeComponent::Tab(new_path);
        }

        DockDragPayload::set(
            ctx,
            DockDragPayload {
                dock_area_id: self.id,
                source_viewport: viewport_for_surface(self.id, torn_surface),
                source_surface: torn_surface,
                tab_path: new_path,
            },
        );

        ctx.request_repaint_of(ViewportId::ROOT);
    }

    pub(in crate::widgets::dock_area) fn finish_ghost_drag(
        &mut self,
        ctx: &Context,
        state: &mut State,
    ) {
        let Some(ghost) = state.ghost_drag.take() else {
            return;
        };

        if !ctx.input(|i| i.key_pressed(Key::Escape)) {
            state.ghost_drag = Some(ghost);
            return;
        }

        self.abort_ghost_drag(ctx, state, ghost);
    }

    /// ESC / cancel: put the tab back and tear down the ghost surface.
    pub(in crate::widgets::dock_area) fn abort_ghost_drag(
        &mut self,
        ctx: &Context,
        state: &mut State,
        ghost: GhostDrag,
    ) {
        let torn_tab = TabPath {
            surface: ghost.torn_surface,
            node: NodeIndex::root(),
            tab: TabIndex(0),
        };
        self.dock_state.move_tab(torn_tab, ghost.restore);

        match ghost.mode {
            GhostDragMode::Native => {
                let viewport_id = WindowState::viewport_id(self.id, ghost.torn_surface);
                ctx.send_viewport_cmd_to(viewport_id, ViewportCommand::Close);
            }
            GhostDragMode::ContainedFloating => {
                // `move_tab` removes the empty surface when applicable.
            }
        }

        state.live_tear_off_surface = None;
        state.dnd = None;
        DockDragPayload::clear(ctx);
        ctx.request_repaint_of(ViewportId::ROOT);
    }

    /// Finalize ghost as a persistent contained floating panel (release outside dock targets).
    pub(in crate::widgets::dock_area) fn finalize_ghost_as_contained_floating(
        &mut self,
        ctx: &Context,
        state: &mut State,
        pointer_screen: Pos2,
    ) {
        let Some(ghost) = state.ghost_drag.take() else {
            return;
        };
        let GhostDragMode::ContainedFloating = ghost.mode else {
            state.ghost_drag = Some(ghost);
            return;
        };

        if let Some(local) =
            self.pointer_screen_to_viewport_local(ctx, ghost.host_viewport, pointer_screen)
        {
            if let Some(ws) = self.dock_state.get_window_state_mut(ghost.torn_surface) {
                ws.set_floating_in_viewport(true);
                ws.set_viewport_local_position_persist(local - ghost.grab_offset);
            }
            self.register_contained_floating(ghost.host_viewport, ghost.torn_surface);
        }

        state.live_tear_off_surface = None;
        state.dnd = None;
        DockDragPayload::clear(ctx);
    }

    pub(in crate::widgets::dock_area) fn update_ghost_follow_pointer(
        &mut self,
        ctx: &Context,
        state: &State,
    ) {
        let Some(ghost) = state.ghost_drag.as_ref() else {
            return;
        };

        let Some(pointer) = state.mv_drag.pointer_global_fallback(ctx) else {
            return;
        };

        match ghost.mode {
            GhostDragMode::Native => {
                let Some(ws) = self.dock_state.get_window_state(ghost.torn_surface) else {
                    return;
                };
                if !ws.uses_native_viewport() || ws.is_floating_in_viewport() {
                    return;
                }
                let viewport_id = WindowState::viewport_id(self.id, ghost.torn_surface);
                ctx.send_viewport_cmd_to(
                    viewport_id,
                    ViewportCommand::OuterPosition(pointer.round()),
                );
            }
            GhostDragMode::ContainedFloating => {
                if let Some(local) =
                    self.pointer_screen_to_viewport_local(ctx, ghost.host_viewport, pointer)
                {
                    if let Some(ws) = self.dock_state.get_window_state_mut(ghost.torn_surface) {
                        ws.set_viewport_local_position_persist(local - ghost.grab_offset);
                    }
                }
            }
        }
    }

    pub(in crate::widgets::dock_area) fn ghost_torn_surface(&self, state: &State) -> Option<SurfaceIndex> {
        state.ghost_drag.as_ref().map(|g| g.torn_surface)
    }

    pub(in crate::widgets::dock_area) fn is_contained_ghost_active(&self, state: &State) -> bool {
        state
            .ghost_drag
            .as_ref()
            .is_some_and(|g| g.mode == GhostDragMode::ContainedFloating)
    }

    fn pointer_screen_to_viewport_local(
        &self,
        ctx: &Context,
        viewport_id: ViewportId,
        pointer_screen: Pos2,
    ) -> Option<Pos2> {
        let inner_min = ctx.input(|i| {
            i.raw
                .viewports
                .get(&viewport_id)?
                .inner_rect
                .map(|r| r.min)
        })?;
        Some(pointer_screen - inner_min.to_vec2())
    }
}
