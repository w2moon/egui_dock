use egui::Context;

use super::drag_and_drop::{DragData, HoverData};

pub(in crate::widgets::dock_area) fn set_drag_data(ctx: &Context, dock_id: egui::Id, data: DragData) {
    ctx.data_mut(|d| d.insert_temp(dock_id.with("drag_data"), Some(data)));
}

pub(in crate::widgets::dock_area) fn take_drag_data(
    ctx: &Context,
    dock_id: egui::Id,
) -> Option<DragData> {
    ctx.data_mut(|d| d.remove_temp(dock_id.with("drag_data")).flatten())
}

pub(in crate::widgets::dock_area) fn set_hover_data(
    ctx: &Context,
    dock_id: egui::Id,
    data: HoverData,
) {
    ctx.data_mut(|d| d.insert_temp(dock_id.with("hover_data"), Some(data)));
}

pub(in crate::widgets::dock_area) fn take_hover_data(
    ctx: &Context,
    dock_id: egui::Id,
) -> Option<HoverData> {
    ctx.data_mut(|d| d.remove_temp(dock_id.with("hover_data")).flatten())
}
