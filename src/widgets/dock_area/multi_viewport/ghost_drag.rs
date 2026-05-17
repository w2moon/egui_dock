use egui::{Context, Key, ViewportCommand, ViewportId};

use crate::{
    dock_area::state::State,
    NodeIndex, SurfaceIndex, TabDestination, TabIndex, TabPath, WindowState,
};

/// Mid-drag ghost tear-off state (tab extracted until drop or ESC).
#[derive(Debug)]
pub struct GhostDrag {
    /// Where to re-insert the tab when the user presses ESC.
    pub restore: TabDestination,
    pub torn_surface: SurfaceIndex,
    pub native_viewport: bool,
}

use crate::DockArea;

impl<Tab> DockArea<'_, Tab> {
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

        let torn_tab = TabPath {
            surface: ghost.torn_surface,
            node: NodeIndex::root(),
            tab: TabIndex(0),
        };
        self.dock_state.move_tab(torn_tab, ghost.restore);

        if ghost.native_viewport {
            let viewport_id = WindowState::viewport_id(self.id, ghost.torn_surface);
            ctx.send_viewport_cmd_to(viewport_id, ViewportCommand::Close);
        }

        state.live_tear_off_surface = None;
        state.dnd = None;
        super::DockDragPayload::clear(ctx);
        ctx.request_repaint_of(ViewportId::ROOT);
    }

    pub(in crate::widgets::dock_area) fn update_torn_viewport_follow_pointer(
        &mut self,
        ctx: &Context,
        state: &State,
    ) {
        let Some(surface) = state
            .ghost_drag
            .as_ref()
            .map(|g| g.torn_surface)
            .or(state.live_tear_off_surface)
        else {
            return;
        };

        let Some(ws) = self.dock_state.get_window_state(surface) else {
            return;
        };
        if !ws.uses_native_viewport() || ws.is_floating_in_viewport() {
            return;
        }

        let Some(pointer) = state.mv_drag.pointer_global_fallback(ctx) else {
            return;
        };

        let viewport_id = WindowState::viewport_id(self.id, surface);
        ctx.send_viewport_cmd_to(
            viewport_id,
            ViewportCommand::OuterPosition(pointer.round()),
        );
    }
}
