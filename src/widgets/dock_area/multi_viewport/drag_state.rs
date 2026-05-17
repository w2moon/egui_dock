use egui::{Context, Pos2, Vec2, ViewportId};

use super::geometry::{pointer_pos_in_global, viewport_under_pointer_global};

/// Integrated pointer state across all viewports (egui_docking-style).
#[derive(Clone, Debug, Default)]
pub struct MultiViewportDragState {
    last_pointer_global: Option<Pos2>,
    last_hovered_viewport: Option<ViewportId>,
    last_interact_update_frame: u64,
    last_delta_integrated_frame: u64,
}

#[allow(dead_code)]
impl MultiViewportDragState {
    pub fn begin_frame(&mut self) {}

    pub fn update_from_ctx(&mut self, ctx: &Context, allow_interact_pos: bool) {
        let frame = ctx.cumulative_pass_nr();

        if allow_interact_pos {
            if let Some(pos) = pointer_pos_in_global(ctx) {
                self.last_pointer_global = Some(pos);
                self.last_interact_update_frame = frame;
                self.last_hovered_viewport = viewport_under_pointer_global(ctx, pos)
                    .or_else(|| Some(ctx.viewport_id()));
            }
        }

        if self.last_interact_update_frame == frame {
            return;
        }

        if let Some(prev) = self.last_pointer_global {
            // Use `i.pixels_per_point` — `ctx.pixels_per_point()` calls `ctx.input` again and deadlocks.
            let delta_points = ctx.input(|i| {
                if let Some(motion) = i.pointer.motion() {
                    let ppp = i.pixels_per_point;
                    if ppp > 0.0 && ppp.is_finite() {
                        motion / ppp
                    } else {
                        Vec2::ZERO
                    }
                } else if allow_interact_pos {
                    i.pointer.delta()
                } else {
                    Vec2::ZERO
                }
            });
            if delta_points != Vec2::ZERO && self.last_delta_integrated_frame != frame {
                self.last_delta_integrated_frame = frame;
                let next = prev + delta_points;
                self.last_pointer_global = Some(next);
                self.last_hovered_viewport = viewport_under_pointer_global(ctx, next)
                    .or(self.last_hovered_viewport);
            }
        }
    }

    pub fn last_pointer_global(&self) -> Option<Pos2> {
        self.last_pointer_global
    }

    pub fn last_hovered_viewport(&self) -> Option<ViewportId> {
        self.last_hovered_viewport
    }

    pub fn pointer_global_fallback(&self, ctx: &Context) -> Option<Pos2> {
        pointer_pos_in_global(ctx).or(self.last_pointer_global)
    }

    pub fn pointer_global_prefer_integrated(&self, ctx: &Context) -> Option<Pos2> {
        self.last_pointer_global
            .or_else(|| pointer_pos_in_global(ctx))
    }
}
