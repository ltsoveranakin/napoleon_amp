use eframe::egui::{Id, Modal, Ui};

pub(super) enum MenuPage {
    Settings
}

impl MenuPage {
    fn render(&mut self, ui: &mut Ui) {
        match self {
            Self::Settings => {

            }
        }
    }
}

pub(super) struct MenuModal {
    page: MenuPage,
}

impl MenuModal {
    pub(super) fn new(page: MenuPage) -> Self {
        Self {
            page
        }
    }

    pub(super) fn render(&mut self, ui: &mut Ui) -> bool {
        let modal = Modal::new(Id::new("menu_modal")).show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    self.render_pagelist(ui);
                });

                ui.vertical(|ui| {
                    self.page.render(ui);
                });

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        return true;
                    }

                    return false;
                }).inner
            }).inner
        });

        modal.should_close() || modal.inner
    }

    fn render_pagelist(&mut self, ui: &mut Ui) {
        if ui.button("Settings").clicked() {
            self.page = MenuPage::Settings;
        }
    }
}
