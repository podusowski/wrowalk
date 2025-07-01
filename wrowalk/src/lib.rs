mod io;
mod mpkwroclaw;
mod places;
mod style;
mod tiles;
mod windows;

use std::collections::BTreeMap;

use egui::{CentralPanel, Color32, Context, FontId, Frame};
use tiles::{providers, Provider, TilesKind};
use walkers::{
    extras::{LabeledSymbol, Places},
    Map, MapMemory,
};

pub struct MyApp {
    providers: BTreeMap<Provider, Vec<TilesKind>>,
    selected_provider: Provider,
    map_memory: MapMemory,
    mpkwroclaw: mpkwroclaw::MpkWroclaw,
}

impl MyApp {
    pub fn new(egui_ctx: Context) -> Self {
        egui_ctx.set_style(style::amoled_friendly());
        egui_material_icons::initialize(&egui_ctx);

        Self {
            providers: providers(egui_ctx.to_owned()),
            selected_provider: Provider::OpenStreetMap,
            map_memory: MapMemory::default(),
            mpkwroclaw: mpkwroclaw::MpkWroclaw::new(egui_ctx),
        }
    }

    fn positions(&self) -> Vec<LabeledSymbol> {
        self.mpkwroclaw
            .vehicles()
            .iter()
            .map(|(_, vehicle)| {
                let position = vehicle.position();
                walkers::extras::LabeledSymbol {
                    position,
                    //label: format!("{}", position.line_name),
                    label: "".to_string(),
                    //symbol: Some(walkers::extras::Symbol::TwoCorners('🚌'.to_string())),
                    symbol: Some(walkers::extras::Symbol::Circle(vehicle.line.clone())),
                    style: walkers::extras::LabeledSymbolStyle {
                        label_corner_radius: 1.,
                        symbol_size: 22.,
                        symbol_background: Color32::BLACK.gamma_multiply(0.8),
                        symbol_color: Color32::WHITE,
                        symbol_font: FontId::proportional(10.),
                        symbol_stroke: egui::Stroke::new(1., Color32::WHITE),
                        ..Default::default()
                    },
                }
            })
            .collect()
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().frame(Frame::NONE).show(ctx, |ui| {
            let my_position = places::wroclaw_glowny();

            let positions = self.positions();

            let tiles = self.providers.get_mut(&self.selected_provider).unwrap();
            let attributions: Vec<_> = tiles
                .iter()
                .map(|tile| tile.as_ref().attribution())
                .collect();

            let mut map = Map::new(None, &mut self.map_memory, my_position)
                .zoom_with_ctrl(false)
                .with_plugin(Places::new(positions));

            // Add layers.
            for (n, tiles) in tiles.iter_mut().enumerate() {
                let transparency = if n == 0 { 1.0 } else { 0.25 };
                map = map.with_layer(tiles.as_mut(), transparency);
            }

            ui.add(map);

            // Show utility windows.
            {
                use windows::*;

                zoom(ui, &mut self.map_memory);
                acknowledge(self, ui, attributions);
            }
        });
    }
}
