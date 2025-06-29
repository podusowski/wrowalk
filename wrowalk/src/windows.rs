use crate::MyApp;
use egui::{Align2, ComboBox, Image, Response, RichText, Ui, Window};
use walkers::{sources::Attribution, MapMemory};

pub fn acknowledge(ui: &Ui, attributions: Vec<Attribution>) {
    Window::new("Acknowledge")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::LEFT_TOP, [10., 10.])
        .show(ui.ctx(), |ui| {
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

pub fn controls(app: &mut MyApp, ui: &Ui) {
    Window::new("Controls")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::RIGHT_TOP, [-10., 10.])
        .fixed_size([150., 150.])
        .show(ui.ctx(), |ui| {
            ui.collapsing("Map", |ui| {
                ComboBox::from_label("Tile Provider")
                    .selected_text(format!("{:?}", app.selected_provider))
                    .show_ui(ui, |ui| {
                        for p in app.providers.keys() {
                            ui.selectable_value(&mut app.selected_provider, *p, format!("{:?}", p));
                        }
                    });
            });
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

                if map_memory.detached().is_some() {
                    if large_material_button(ui, "\u{e55c}").clicked() {
                        map_memory.follow_my_position();
                    }
                }
            });
        });
}
