use std::cell::Cell;

use egui::{
    vec2, CentralPanel, Color32, Context, Frame, Rect, WidgetText,
};

use crate::{
    dock_area::{state::State, tab_removal::TabRemoval},
    utils::fade_visuals,
    DockArea, Style, SurfaceIndex, TabViewer, WindowState,
};

impl<Tab> DockArea<'_, Tab> {
    /// Renders every detached surface as a native [`egui::viewport`] window.
    pub(super) fn show_viewport_surfaces(
        &mut self,
        ctx: &Context,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        state: &mut State,
        fade_style: Option<(&Style, f32, SurfaceIndex)>,
    ) {
        let surface_indices: Vec<SurfaceIndex> = self
            .dock_state
            .valid_surface_indices()
            .iter()
            .copied()
            .filter(|index| !index.is_main())
            .collect();

        let mut surfaces_to_close = Vec::new();

        for surf_index in surface_indices {
            let use_native = self
                .dock_state
                .get_window_state(surf_index)
                .is_some_and(|ws| ws.uses_native_viewport() && !ws.is_floating_in_viewport());
            if !use_native {
                continue;
            }
            if self.show_viewport_surface(
                ctx,
                surf_index,
                tab_viewer,
                state,
                fade_style,
            ) {
                surfaces_to_close.push(surf_index);
            }
        }

        for surf_index in surfaces_to_close {
            self.to_remove.push(TabRemoval::Window(surf_index));
        }
    }

    /// Returns `true` when the native viewport requested to close.
    fn show_viewport_surface(
        &mut self,
        ctx: &Context,
        surf_index: SurfaceIndex,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
        state: &mut State,
        fade_style: Option<(&Style, f32, SurfaceIndex)>,
    ) -> bool {
        let title = self.window_surface_title(ctx, surf_index, tab_viewer);
        let tab_count = self.window_surface_tab_count(surf_index);

        let (fade_factor, fade_style) = match fade_style {
            Some((style, factor, surface_index)) => {
                if surface_index == surf_index {
                    (1.0, None)
                } else {
                    (factor, Some((style, factor)))
                }
            }
            None => (1.0, None),
        };

        let tab_bar_height = self.style.as_ref().unwrap().tab_bar.height;
        let minimized = self
            .dock_state
            .get_window_state(surf_index)
            .unwrap()
            .is_minimized();

        let inner_size_override = if minimized {
            Some(vec2(320.0, tab_bar_height))
        } else if self.dock_state[surf_index].is_collapsed() {
            let height =
                self.dock_state[surf_index].collapsed_leaf_count() as f32 * tab_bar_height;
            Some(vec2(320.0, height))
        } else {
            None
        };

        let builder = self
            .dock_state
            .get_window_state_mut(surf_index)
            .unwrap()
            .create_viewport_builder(
                title.text().to_string(),
                self.native_window_decorations,
                inner_size_override,
            );

        let viewport_id = WindowState::viewport_id(self.id, surf_index);
        let close_requested = Cell::new(false);
        let outer_rect = Cell::new(None::<Rect>);

        ctx.show_viewport_immediate(viewport_id, builder, |ctx, class| {
            if ctx.input(|i| i.viewport().close_requested()) {
                close_requested.set(true);
                return;
            }

            outer_rect.set(ctx.input(|i| i.viewport().outer_rect));

            let mut frame = Frame::window(ctx.style());
            if fade_factor != 1.0 {
                frame.fill = frame.fill.linear_multiply(fade_factor);
                frame.stroke.color = frame.stroke.color.linear_multiply(fade_factor);
                frame.shadow.color = frame.shadow.color.linear_multiply(fade_factor);
            }

            let panel = |ui: &mut egui::Ui| {
                state.mv_drag.update_from_ctx(ui.ctx(), true);
                if fade_factor != 1.0 {
                    fade_visuals(ui.visuals_mut(), fade_factor);
                }
                self.show_contained_floating_surfaces(ui, viewport_id, tab_viewer, state);
                if minimized {
                    self.minimized_body(
                        ui,
                        surf_index,
                        fade_style.map(|(style, _)| style),
                        title.clone(),
                        tab_count,
                    );
                } else {
                    self.render_nodes(ui, tab_viewer, state, surf_index, fade_style);
                }
            };

            let _ = class;
            CentralPanel::default()
                .frame(
                    frame
                        .inner_margin(0.0)
                        .fill(Color32::TRANSPARENT),
                )
                .show_inside(ctx, panel);
        });

        if let Some(window_state) = self.dock_state.get_window_state_mut(surf_index) {
            window_state.update_screen_rect_from_viewport(outer_rect.get());
        }

        close_requested.get()
    }

    pub(super) fn window_surface_title(
        &mut self,
        ctx: &Context,
        surf_index: SurfaceIndex,
        tab_viewer: &mut impl TabViewer<Tab = Tab>,
    ) -> WidgetText {
        let node_id = self.dock_state[surf_index]
            .focused_leaf()
            .unwrap_or_else(|| {
                for node_index in self.dock_state[surf_index].breadth_first_index_iter() {
                    if self.dock_state[surf_index][node_index].is_leaf() {
                        return node_index;
                    }
                }
                unreachable!("a window surface should never be empty")
            });
        let leaf = self.dock_state[surf_index][node_id].get_leaf_mut().unwrap();
        tab_viewer
            .title(&mut leaf.tabs[leaf.active.0])
            .color(ctx.global_style().visuals.widgets.noninteractive.fg_stroke.color)
    }

    fn window_surface_tab_count(&self, surf_index: SurfaceIndex) -> usize {
        let mut tab_count = 0;
        for node_index in self.dock_state[surf_index].breadth_first_index_iter() {
            if self.dock_state[surf_index][node_index].is_leaf() {
                tab_count += self.dock_state[surf_index][node_index].tabs_count();
            }
        }
        tab_count
    }
}
