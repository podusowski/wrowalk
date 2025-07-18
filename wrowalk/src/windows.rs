use crate::MyApp;
use egui::{Align2, ComboBox, Image, Response, RichText, Ui, Window};
use walkers::{sources::Attribution, MapMemory};

pub fn acknowledge(app: &mut MyApp, ui: &Ui, attributions: Vec<Attribution>) {
    Window::new("Acknowledge")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::LEFT_TOP, [10., 10.])
        .show(ui.ctx(), |ui| {
            ui.label(format!("Tracking {} vehicles.", app.positions().len()));

            ComboBox::from_id_salt("Tile Provider")
                .selected_text(format!("{:?}", app.selected_provider))
                .show_ui(ui, |ui| {
                    for p in app.providers.keys() {
                        ui.selectable_value(&mut app.selected_provider, *p, format!("{p:?}"));
                    }
                });

            for attribution in attributions {
                ui.horizontal(|ui| {
                    if let Some(logo) = attribution.logo_light {
                        ui.add(Image::new(logo).max_height(30.0).max_width(80.0));
                    }
                    ui.hyperlink_to(attribution.text, attribution.url);
                });
            }
        });
}

pub fn large_material_button(ui: &mut Ui, text: &str) -> Response {
    ui.button(RichText::new(text).size(24.0))
}

/// Simple GUI to zoom in and out.
pub fn zoom(ui: &Ui, map_memory: &mut MapMemory) {
    Window::new("Map")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::LEFT_BOTTOM, [10., -10.])
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                if large_material_button(ui, "\u{e145}").clicked() {
                    let _ = map_memory.zoom_in();
                }

                if large_material_button(ui, "\u{e15b}").clicked() {
                    let _ = map_memory.zoom_out();
                }

                if map_memory.detached().is_some()
                    && large_material_button(ui, "\u{e55c}").clicked()
                {
                    map_memory.follow_my_position();
                }
            });
        });
}
