use std::sync::{Arc, Mutex};

use egui::{Color32, FontId};
use serde::Deserialize;
use walkers::extras::LabeledSymbol;

pub async fn fetch_vehicles() -> Vec<Vehicle> {
    let url =
        "https://www.wroclaw.pl/open-data/datastore/dump/a9b3841d-e977-474e-9e86-8789e470a85a";
    let result = reqwest::get(url).await.unwrap();
    let bytes = result.bytes().await.unwrap();

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(bytes.as_ref());

    rdr.deserialize()
        .map(|record| {
            let record: Vehicle = record.unwrap();
            record
        })
        .filter(Vehicle::sane)
        .collect()
}

#[derive(Deserialize, Debug)]
pub struct Vehicle {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "Nr_Boczny")]
    pub fleet_number: String,
    #[serde(rename = "Nr_Rej")]
    pub registration_number: String,
    #[serde(rename = "Brygada")]
    pub brigade: String,
    #[serde(rename = "Nazwa_Linii")]
    pub line_name: String,
    #[serde(rename = "Ostatnia_Pozycja_Szerokosc")]
    pub latitude: f64,
    #[serde(rename = "Ostatnia_Pozycja_Dlugosc")]
    pub longitude: f64,
    #[serde(rename = "Data_Aktualizacji")]
    pub last_update: String,
}

impl Vehicle {
    /// Does this record even make sense.
    fn sane(position: &Vehicle) -> bool {
        position.line_name != "None" && position.line_name != ""
    }
}

impl From<Vehicle> for walkers::extras::LabeledSymbol {
    fn from(position: Vehicle) -> Self {
        walkers::extras::LabeledSymbol {
            position: walkers::lat_lon(position.latitude, position.longitude),
            //label: format!("{}", position.line_name),
            label: "".to_string(),
            //symbol: Some(walkers::extras::Symbol::TwoCorners('🚌'.to_string())),
            symbol: Some(walkers::extras::Symbol::Circle(position.line_name)),
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
    }
}

pub struct MpkWroclaw {
    #[allow(dead_code)]
    runtime: crate::io::Runtime,

    positions: Arc<Mutex<Vec<LabeledSymbol>>>,
}

impl MpkWroclaw {
    pub fn new(egui_ctx: egui::Context) -> Self {
        let positions = Arc::new(Mutex::new(Vec::new()));
        let positions_clone = positions.clone();

        Self {
            positions,
            runtime: crate::io::Runtime::new(async move {
                loop {
                    log::debug!("Tick.");
                    let positions = fetch_vehicles().await;
                    log::debug!("Fetched positions: {:#?}", positions);
                    {
                        let mut positions_lock = positions_clone.lock().unwrap();
                        *positions_lock = positions.into_iter().map(LabeledSymbol::from).collect();
                        log::debug!("Updated positions: {}", positions_lock.len());
                    }
                    egui_ctx.request_repaint();
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }),
        }
    }

    pub fn positions(&self) -> Vec<LabeledSymbol> {
        self.positions.lock().unwrap().clone()
    }
}
