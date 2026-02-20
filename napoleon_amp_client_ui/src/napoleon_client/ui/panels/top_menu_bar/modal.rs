use eframe::egui::{Id, Modal, Ui};

pub(super) struct MenuModal {}

impl MenuModal {
    pub(super) fn new() -> Self {
        Self {}
    }

    pub(super) fn render(&self, ui: &mut Ui) -> bool {
        let modal = Modal::new(Id::new("menu_modal")).show(ui.ctx(), |ui| {});

        modal.should_close()
    }
}
