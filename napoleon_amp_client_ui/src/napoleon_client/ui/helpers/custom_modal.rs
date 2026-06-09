use crate::napoleon_client::ui::helpers::close_ui;
use crate::napoleon_client::ui::panels::CloseResult;
use eframe::egui;
use eframe::egui::{Modal, ModalResponse, Ui};

pub(crate) fn custom_modal(
    ui: &mut Ui,
    title: &'static str,
    add_contents: impl FnOnce(&mut Ui),
) -> ModalResponse<CloseResult> {
    Modal::new(egui::Id::new(title)).show(ui.ctx(), |ui| {
        ui.heading(title);
        ui.separator();

        add_contents(ui);

        close_ui(ui)
    })
}
