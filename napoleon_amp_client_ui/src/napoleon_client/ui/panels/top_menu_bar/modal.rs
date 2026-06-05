use crate::napoleon_client::ui::panels::CloseResult;
use eframe::egui::{Id, Modal, Slider, Ui};
use napoleon_amp_core::content::SaveData;
use napoleon_amp_core::instance::NapoleonInstance;

pub(super) enum MenuPage {
    Settings,
}

impl MenuPage {
    fn render(&mut self, ui: &mut Ui, napoleon_instance: &mut NapoleonInstance) {
        match self {
            Self::Settings => {
                ui.label("Inactive render timeout (ms):");
                ui.add(Slider::new(
                    &mut napoleon_instance
                        .get_client_settings()
                        .inner
                        .inactive_render_timeout_ms,
                    100..=8_000,
                ));
            }
        }
    }
}

pub(super) struct MenuModal {
    page: MenuPage,
}

impl MenuModal {
    pub(super) fn new(page: MenuPage) -> Self {
        Self { page }
    }

    pub(super) fn render(&mut self, ui: &mut Ui, napoleon_instance: &mut NapoleonInstance) -> bool {
        let modal = Modal::new(Id::new("menu_modal")).show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    self.render_pagelist(ui);
                });

                ui.vertical(|ui| {
                    self.page.render(ui, napoleon_instance);

                    ui.horizontal(|ui| {
                        if ui.button("Ok").clicked() {
                            return CloseResult::SaveAndClose;
                        }

                        if ui.button("Cancel").clicked() {
                            return CloseResult::CloseWithoutSaving;
                        }

                        return CloseResult::KeepOpen;
                    })
                    .inner
                })
                .inner
            })
            .inner
        });

        match modal.inner {
            CloseResult::KeepOpen => {}

            CloseResult::CloseWithoutSaving => {}

            CloseResult::SaveAndClose => {
                napoleon_instance
                    .get_client_settings()
                    .save_data(())
                    .expect("Failed save client settings");
            }
        };

        modal.should_close() || modal.inner.should_close()
    }

    fn render_pagelist(&mut self, ui: &mut Ui) {
        if ui.button("Settings").clicked() {
            self.page = MenuPage::Settings;
        }
    }
}
