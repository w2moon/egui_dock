use egui::{Context, Pos2, Rect, Vec2};

use crate::{
    dock_area::{
        drag_and_drop::{DragData, DragDropState, HoverData, TreeComponent},
        state::State,
    },
    DockArea, Node, NodePath, OverlayType, SurfaceIndex, TabDestination,
    TabPath, TabViewer,
};

use super::pending_drop::PendingDrop;

use super::geometry::viewport_for_surface;
use super::payload::DockDragPayload;

impl<Tab> DockArea<'_, Tab> {
    pub(in crate::widgets::dock_area) fn sync_drag_payload_from_tab_drag(
        &self,
        ctx: &Context,
        tab_path: TabPath,
    ) {
        DockDragPayload::set(
            ctx,
            DockDragPayload {
                dock_area_id: self.id,
                source_viewport: viewport_for_surface(self.id, tab_path.surface),
                source_surface: tab_path.surface,
                tab_path,
            },
        );
    }

    /// Applies a drop using [`DockDragPayload`] when the normal [`State::dnd`] path did not run.
    pub(in crate::widgets::dock_area) fn apply_cross_viewport_drop(
        &mut self,
        ctx: &Context,
        state: &mut State,
        destination: Option<TabDestination>,
    ) {
        let Some(payload) = DockDragPayload::take(ctx) else {
            return;
        };

        if payload.dock_area_id != self.id {
            DockDragPayload::set(ctx, (*payload).clone());
            return;
        }

        if let Some(destination) = destination {
            if let TabDestination::Window(rect) = destination {
                if let Some(ghost) = state.ghost_drag.as_ref() {
                    match ghost.mode {
                        super::ghost_drag::GhostDragMode::ContainedFloating => {
                            self.finalize_ghost_as_contained_floating(ctx, state, rect.min);
                            return;
                        }
                        super::ghost_drag::GhostDragMode::Native => {
                            if let Some(ws) =
                                self.dock_state.get_window_state_mut(ghost.torn_surface)
                            {
                                ws.set_position(rect.min);
                            }
                            state.ghost_drag = None;
                            state.live_tear_off_surface = None;
                            return;
                        }
                    }
                }
                if self.multi_viewport_options.tear_off_to_floating_on_ctrl
                    && ctx.input(|i| i.modifiers.ctrl)
                {
                    self.tear_off_to_floating_panel(ctx, payload.tab_path);
                    return;
                }
                let use_contained = self.multi_viewport_options.contained_window_on_ctrl
                    && ctx.input(|i| i.modifiers.ctrl);
                if use_contained {
                    let pointer = self
                        .pointer_screen_for_dock(ctx, state)
                        .unwrap_or(Pos2::ZERO);
                    let size = self.dock_state[payload.tab_path.node_path()]
                        .rect()
                        .map(|r| r.size())
                        .unwrap_or(Vec2::new(320.0, 240.0));
                    let surface = self.dock_state.detach_tab(
                        payload.tab_path,
                        Rect::from_min_size(pointer, size),
                    );
                    if let Some(ws) = self.dock_state.get_window_state_mut(surface) {
                        ws.set_native_viewport(false);
                    }
                    return;
                }
            }
            self.dock_state.move_tab(payload.tab_path, destination);
        }
    }

    pub(in crate::widgets::dock_area) fn clear_drag_payload_if_ours(&self, ctx: &Context) {
        if DockDragPayload::get(ctx).is_some_and(|p| p.dock_area_id == self.id) {
            DockDragPayload::take(ctx);
        }
    }

    pub(in crate::widgets::dock_area) fn resolve_hover_for_cross_viewport(
        &self,
        ctx: &Context,
        state: &State,
        pointer_screen: Pos2,
    ) -> Option<HoverData> {
        let ghost_surface = self.ghost_torn_surface(state);

        let mut best: Option<(f32, crate::dock_area::state::DockRectHit)> = None;
        for hit in &state.dock_rects_screen {
            if ghost_surface == Some(hit.surface) {
                continue;
            }
            if !hit.rect.contains(pointer_screen) {
                continue;
            }
            let area = hit.rect.area();
            if best.as_ref().is_none_or(|(best_area, _)| area < *best_area) {
                best = Some((area, *hit));
            }
        }
        let hit = best?.1;
        let local_rect = self.screen_rect_to_local_leaf(ctx, hit.surface, hit.rect)?;
        Some(HoverData {
            rect: local_rect,
            dst: TreeComponent::Node(NodePath {
                surface: hit.surface,
                node: hit.node,
            }),
            tab: None,
        })
    }

    pub(in crate::widgets::dock_area) fn pointer_screen_for_dock(
        &self,
        ctx: &Context,
        state: &State,
    ) -> Option<Pos2> {
        state
            .mv_drag
            .pointer_global_fallback(ctx)
            .or_else(|| super::geometry::pointer_latest_in_screen(ctx))
    }

    pub(in crate::widgets::dock_area) fn apply_pending_drop(
        &mut self,
        ctx: &Context,
        state: &mut State,
        tab_viewer: &mut impl crate::TabViewer<Tab = Tab>,
    ) {
        let Some(pending) = state.pending_drop.take() else {
            return;
        };

        match pending {
            PendingDrop::Tab { destination } => {
                if state.dnd.is_some() {
                    if destination.is_none() && self.is_contained_ghost_active(state) {
                        if let Some(pointer) = self.pointer_screen_for_dock(ctx, state) {
                            self.finalize_ghost_as_contained_floating(ctx, state, pointer);
                        }
                    } else {
                        self.apply_tab_drop(destination, state, tab_viewer, ctx);
                    }
                    self.clear_drag_payload_if_ours(ctx);
                }
            }
            PendingDrop::CrossViewport {
                fallback_destination,
            } => {
                let destination = self
                    .resolve_cross_viewport_drop_destination(ctx, state, tab_viewer)
                    .or(fallback_destination);
                self.apply_cross_viewport_drop(ctx, state, destination);
            }
        }
        state.live_tear_off_surface = None;
    }

    fn pointer_local_on_surface(
        &self,
        ctx: &Context,
        surface: SurfaceIndex,
        pointer_screen: Pos2,
    ) -> Option<Pos2> {
        let viewport_id = viewport_for_surface(self.id, surface);
        let inner_min = ctx.input(|i| {
            i.raw
                .viewports
                .get(&viewport_id)?
                .inner_rect
                .map(|r| r.min)
        })?;
        Some(pointer_screen - inner_min.to_vec2())
    }

    fn resolve_cross_viewport_overlay_destination(
        &mut self,
        ctx: &Context,
        state: &State,
        tab_viewer: &impl TabViewer<Tab = Tab>,
        pointer_screen: Pos2,
    ) -> Option<TabDestination> {
        let hover = self.resolve_hover_for_cross_viewport(ctx, state, pointer_screen)?;

        let drag = if let Some(dnd) = &state.dnd {
            dnd.drag.clone()
        } else {
            let payload = DockDragPayload::get(ctx)?;
            if payload.dock_area_id != self.id {
                return None;
            }
            let tab_path = payload.tab_path;
            let rect = self.dock_state[tab_path.node_path()]
                .rect()
                .unwrap_or(Rect::NOTHING);
            DragData {
                src: TreeComponent::Tab(tab_path),
                rect,
            }
        };

        let surface = hover.dst.surface_address();
        let pointer_local = self.pointer_local_on_surface(ctx, surface, pointer_screen)?;

        let mut dnd = DragDropState {
            hover,
            drag,
            pointer: pointer_local,
            locked: None,
        };

        let style = self.style.as_ref()?;
        let allowed_splits = self.allowed_splits;

        let allowed_in_window = match dnd.drag.src {
            TreeComponent::Tab(path) => {
                let Node::Leaf(leaf) = &mut self.dock_state[path.node_path()] else {
                    return None;
                };
                tab_viewer.allowed_in_windows(&mut leaf.tabs[path.tab.0])
            }
            _ => return None,
        };

        let window_bounds = self.window_bounds.unwrap_or(ctx.content_rect());
        let overlay_id = self.id.with("cross_viewport_overlay");
        let pointer_screen_opt = Some(pointer_screen);

        match style.overlay.overlay_type {
            OverlayType::HighlightedAreas | OverlayType::Widgets => dnd.resolve_traditional_ctx(
                ctx,
                overlay_id,
                style,
                allowed_splits,
                allowed_in_window,
                window_bounds,
                true,
                pointer_screen_opt,
            ),
        }
    }

    fn screen_rect_to_local_leaf(
        &self,
        ctx: &Context,
        surface: SurfaceIndex,
        screen_rect: Rect,
    ) -> Option<Rect> {
        let viewport_id = viewport_for_surface(self.id, surface);
        let inner_min = ctx.input(|i| {
            i.raw
                .viewports
                .get(&viewport_id)?
                .inner_rect
                .map(|r| r.min)
        })?;
        Some(screen_rect.translate(-inner_min.to_vec2()))
    }

    pub(in crate::widgets::dock_area) fn tear_off_to_floating_panel(
        &mut self,
        ctx: &Context,
        tab_path: TabPath,
    ) {
        let pointer = super::geometry::pointer_pos_in_global(ctx).unwrap_or(Pos2::ZERO);
        let viewport_id = viewport_for_surface(self.id, tab_path.surface);
        let local_pos = ctx.input(|i| {
            let inner = i.raw.viewports.get(&viewport_id)?.inner_rect?;
            Some(pointer - inner.min.to_vec2())
        });

        let size = self.dock_state[tab_path.node_path()]
            .rect()
            .map(|r| r.size())
            .unwrap_or(egui::Vec2::new(320.0, 240.0));

        let surface = self.dock_state.detach_tab(
            tab_path,
            egui::Rect::from_min_size(pointer, size),
        );

        if let Some(ws) = self.dock_state.get_window_state_mut(surface) {
            ws.set_floating_in_viewport(true);
            if let Some(pos) = local_pos {
                ws.set_viewport_local_position(pos);
            }
        }

        DockDragPayload::clear(ctx);
        ctx.request_repaint();
    }

    /// Resolves where a tab should land when the pointer is released (cross-viewport path).
    pub(in crate::widgets::dock_area) fn resolve_cross_viewport_drop_destination(
        &mut self,
        ctx: &Context,
        state: &State,
        tab_viewer: &impl TabViewer<Tab = Tab>,
    ) -> Option<TabDestination> {
        let pointer = self.pointer_screen_for_dock(ctx, state)?;
        let shift_blocks_dock = self.multi_viewport_options.disable_docking_while_shift
            && ctx.input(|i| i.modifiers.shift);

        if !shift_blocks_dock {
            if let Some(dest) =
                self.resolve_cross_viewport_overlay_destination(ctx, state, tab_viewer, pointer)
            {
                return Some(dest);
            }
        }

        if self.multi_viewport_options.detach_on_alt && ctx.input(|i| i.modifiers.alt) {
            return Some(TabDestination::Window(Rect::from_min_size(
                pointer,
                Vec2::new(320.0, 240.0),
            )));
        }

        Some(TabDestination::Window(Rect::from_min_size(
            pointer,
            Vec2::new(320.0, 240.0),
        )))
    }
}
