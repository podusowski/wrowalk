mod io;
mod local_tiles;
mod places;
mod plugins;
mod tiles;
mod windows;

use std::collections::BTreeMap;

use egui::{CentralPanel, Context, Frame};
use tiles::{providers, Provider, TilesKind};
use walkers::{Map, MapMemory};

pub struct MyApp {
    providers: BTreeMap<Provider, Vec<TilesKind>>,
    selected_provider: Provider,
    map_memory: MapMemory,

    #[allow(dead_code)]
    runtime: io::Runtime,
}

impl MyApp {
    pub fn new(egui_ctx: Context) -> Self {
        Self {
            providers: providers(egui_ctx.to_owned()),
            selected_provider: Provider::OpenStreetMap,
            map_memory: MapMemory::default(),
            runtime: io::Runtime::new(async {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    log::info!("Tick.");
                }
            }),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().frame(Frame::NONE).show(ctx, |ui| {
            let my_position = places::wroclaw_glowny();

            let tiles = self.providers.get_mut(&self.selected_provider).unwrap();
            let attributions: Vec<_> = tiles
                .iter()
                .map(|tile| tile.as_ref().attribution())
                .collect();

            let mut map = Map::new(None, &mut self.map_memory, my_position)
                .zoom_with_ctrl(false)
                .with_plugin(plugins::places());

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
                go_to_my_position(ui, &mut self.map_memory);
                controls(self, ui);
                acknowledge(ui, attributions);
            }
        });
    }
}
