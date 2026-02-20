mod modal;

use crate::napoleon_client::ui::panels::top_menu_bar::modal::MenuModal;
use eframe::egui::Ui;

pub(crate) struct TopMenuBar {
    menu_modal: Option<MenuModal>,
}

impl TopMenuBar {
    pub(crate) fn new() -> Self {
        Self { menu_modal: None }
    }

    pub(crate) fn render(&mut self, ui: &mut Ui) {
        self.render_menu_bar(ui);

        let mut should_close = false;

        if let Some(menu_modal) = &self.menu_modal {
            should_close = menu_modal.render(ui);
        }

        if should_close {
            self.menu_modal.take();
        }
    }

    fn render_menu_bar(&mut self, ui: &mut Ui) {
        ui.menu_button("File", |_ui| {});

        ui.menu_button("Edit", |ui| {
            if ui.button("Settings").clicked() {
                self.menu_modal = Some(MenuModal::new());
            }
        });
    }
}
