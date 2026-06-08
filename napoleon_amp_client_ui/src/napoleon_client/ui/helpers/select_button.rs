use derive_enum_all_values::AllValues;
use eframe::egui::Ui;

/// A menu select button, returns true if the value was mutated

pub(crate) fn select_button<V, F, R>(
    ui: &mut Ui,
    button_text: &'static str,
    value: &V,
    on_change: F,
) -> Option<R>
where
    V: ToString + AllValues + 'static,
    F: Fn(&V) -> R,
{
    let mut return_value = None;

    ui.menu_button(format!("{}: {}", button_text, value.to_string()), |ui| {
        for value_it in V::all_values() {
            if ui.button(value_it.to_string()).clicked() {
                return_value = Some(on_change(value_it));
            }
        }
    });

    return_value
}

pub(crate) fn select_button_mut<V>(ui: &mut Ui, button_text: &'static str, value: &mut V)
where
    V: ToString + AllValues + Clone + 'static,
{
    if let Some(new_value) = select_button(ui, button_text, value, |new_value| new_value.clone()) {
        *value = new_value;
    }
}
