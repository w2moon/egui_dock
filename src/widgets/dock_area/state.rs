use egui::{Context, Id, Pos2, Rect};

use super::drag_and_drop::{DragData, DragDropState, HoverData};
use crate::{Style, SurfaceIndex};

#[derive(Clone, Debug, Default)]
pub(super) struct State {
    pub drag_start: Option<Pos2>,
    pub last_hover_pos: Option<Pos2>,
    pub dnd: Option<DragDropState>,
    pub window_fade: Option<(f64, SurfaceIndex)>,
    /// Leaf rects in screen space, rebuilt each frame (multi-viewport).
    pub dock_rects_screen: Vec<(SurfaceIndex, Rect)>,
    /// Set when [`super::multi_viewport::MultiViewportOptions::live_tear_off`] detaches mid-drag.
    pub live_tear_off_surface: Option<SurfaceIndex>,
}

impl State {
    #[inline(always)]
    pub(super) fn load(ctx: &Context, id: Id) -> Self {
        ctx.data_mut(|d| d.get_temp(id)).unwrap_or(Self {
            drag_start: None,
            last_hover_pos: None,
            dnd: None,
            window_fade: None,
            dock_rects_screen: Vec::new(),
            live_tear_off_surface: None,
        })
    }

    #[inline(always)]
    pub(super) fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|d| d.insert_temp(id, self));
    }

    pub(super) fn reset_drag(&mut self) {
        self.dnd = None;
        self.window_fade = None;
        self.drag_start = None;
        self.live_tear_off_surface = None;
    }

    #[inline]
    pub(super) fn clear_dock_rects_screen(&mut self) {
        self.dock_rects_screen.clear();
    }

    pub(super) fn push_dock_rect_screen(&mut self, surface: SurfaceIndex, rect: Rect) {
        self.dock_rects_screen.push((surface, rect));
    }

    pub(super) fn set_drag_and_drop(
        &mut self,
        drag: DragData,
        drop: HoverData,
        ctx: &Context,
        style: &Style,
    ) {
        if !self.is_drag_drop_locked(ctx, style) {
            self.dnd = Some(DragDropState {
                hover: drop,
                drag,
                pointer: ctx.pointer_hover_pos().unwrap_or(Pos2::ZERO),
                locked: None,
            })
        }
    }

    #[inline(always)]
    fn is_drag_drop_locked(&self, ctx: &Context, style: &Style) -> bool {
        self.dnd
            .as_ref()
            .is_some_and(|drag_drop_state| drag_drop_state.is_locked(style, ctx))
    }
}
