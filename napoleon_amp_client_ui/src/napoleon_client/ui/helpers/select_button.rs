use derive_enum_all_values::AllValues;
use eframe::egui::Ui;

/// A menu select button, returns true if the value was mutated

pub(crate) fn select_button<V, F>(ui: &mut Ui, button_text: &'static str, value: &V, on_change: F)
where
    V: ToString + AllValues + Clone + 'static,
    F: Fn(&V),
{
    ui.menu_button(format!("{}: {}", button_text, value.to_string()), |ui| {
        for value_it in V::all_values() {
            if ui.button(value_it.to_string()).clicked() {
                on_change(value_it);
            }
        }
    });
}
