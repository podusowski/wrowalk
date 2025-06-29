mod io;
mod mpkwroclaw;
mod places;
mod tiles;
mod windows;

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use egui::{CentralPanel, Context, Frame};
use tiles::{providers, Provider, TilesKind};
use walkers::{
    extras::{LabeledSymbol, Places},
    Map, MapMemory,
};

pub struct MyApp {
    providers: BTreeMap<Provider, Vec<TilesKind>>,
    selected_provider: Provider,
    map_memory: MapMemory,
    positions: Arc<Mutex<Vec<LabeledSymbol>>>,

    #[allow(dead_code)]
    runtime: io::Runtime,
}

impl MyApp {
    pub fn new(egui_ctx: Context) -> Self {
        let positions = Arc::new(Mutex::new(Vec::new()));
        let positions_clone = positions.clone();

        Self {
            providers: providers(egui_ctx.to_owned()),
            selected_provider: Provider::OpenStreetMap,
            map_memory: MapMemory::default(),
            positions,
            runtime: io::Runtime::new(async move {
                loop {
                    log::debug!("Tick.");
                    let positions = mpkwroclaw::fetch_positions().await;
                    log::debug!("Fetched positions: {:#?}", positions);
                    {
                        let mut positions_lock = positions_clone.lock().unwrap();
                        *positions_lock = positions.into_iter().map(LabeledSymbol::from).collect();
                        log::debug!("Updated positions: {}", positions_lock.len());
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
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

            let positions = self.positions.lock().unwrap().clone();
            log::debug!("Positions: {}", positions.len());

            let mut map = Map::new(None, &mut self.map_memory, my_position)
                .zoom_with_ctrl(false)
                .with_plugin(Places::new(self.positions.lock().unwrap().clone()));

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
