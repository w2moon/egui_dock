use egui::{Context, Pos2, Rect, Vec2};

use crate::{
    dock_area::{
        drag_and_drop::{HoverData, TreeComponent},
        state::State,
    },
    DockArea, NodeIndex, NodePath, SurfaceIndex, TabDestination, TabPath,
};

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
            if let TabDestination::Window(_) = destination {
                if self.multi_viewport_options.tear_off_to_floating_on_ctrl
                    && ctx.input(|i| i.modifiers.ctrl)
                {
                    self.tear_off_to_floating_panel(ctx, payload.tab_path);
                    return;
                }
                let use_contained = self.multi_viewport_options.contained_window_on_ctrl
                    && ctx.input(|i| i.modifiers.ctrl);
                if use_contained {
                    let pointer =
                        super::geometry::pointer_latest_in_screen(ctx).unwrap_or(Pos2::ZERO);
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
        for &(surface, rect) in &state.dock_rects_screen {
            if !rect.contains(pointer_screen) {
                continue;
            }
            if let Some(local_rect) = self.screen_rect_to_local_leaf(ctx, surface, rect) {
                return Some(HoverData {
                    rect: local_rect,
                    dst: TreeComponent::Node(NodePath {
                        surface,
                        node: NodeIndex::root(),
                    }),
                    tab: None,
                });
            }
        }
        None
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
        let pointer = super::geometry::pointer_latest_in_screen(ctx).unwrap_or(Pos2::ZERO);
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
        &self,
        ctx: &Context,
        state: &State,
    ) -> Option<TabDestination> {
        let pointer = super::geometry::pointer_latest_in_screen(ctx)?;
        let shift_blocks_dock = self.multi_viewport_options.disable_docking_while_shift
            && ctx.input(|i| i.modifiers.shift);

        if !shift_blocks_dock {
            if let Some(hover) = self.resolve_hover_for_cross_viewport(ctx, state, pointer) {
                return Some(hover.dst.as_tab_destination());
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
