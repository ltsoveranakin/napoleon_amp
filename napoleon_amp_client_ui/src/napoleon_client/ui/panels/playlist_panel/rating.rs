use eframe::egui::{Image, InnerResponse, Sense, Ui, Vec2, include_image};
use napoleon_amp_core::content::song::song_data::MAX_RATING;

pub(super) fn render_rating(ui: &mut Ui, rating: u8) -> InnerResponse<Option<u8>> {
    ui.horizontal(|ui| {
        let mut updated_rating = None;

        let mut i_rating = rating as i8;

        ui.style_mut().spacing.item_spacing.x = 2.;

        for star_index in 0..MAX_RATING {
            let image_source = if i_rating > 0 {
                i_rating -= 1;
                include_image!("../../../../../../assets/sprites/star_full1.png")
            } else {
                include_image!("../../../../../../assets/sprites/star_empty3.png")
            };

            let star_button = ui.add(
                Image::new(image_source)
                    .sense(Sense::click())
                    .max_size(Vec2::splat(10.)),
            );

            if star_button.clicked() {
                updated_rating = Some(star_index as u8 + 1);
            }

            if star_button.secondary_clicked() {
                updated_rating = Some(0);
            }
        }

        updated_rating
    })
}
