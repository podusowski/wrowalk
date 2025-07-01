use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::sleep;

async fn fetch_vehicles() -> Vec<RawVehicleRecord> {
    let url =
        "https://www.wroclaw.pl/open-data/datastore/dump/a9b3841d-e977-474e-9e86-8789e470a85a";
    let result = reqwest::get(url).await.unwrap();
    let bytes = result.bytes().await.unwrap();

    csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(bytes.as_ref())
        .deserialize()
        .filter_map(|record| {
            let record: RawVehicleRecord = record.ok()?;
            if record.sane() {
                Some(record)
            } else {
                None
            }
        })
        .collect()
}

#[derive(Deserialize, Debug, Clone)]
#[allow(unused)]
struct RawVehicleRecord {
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

impl RawVehicleRecord {
    /// Does this record even make sense.
    fn sane(&self) -> bool {
        self.line_name != "None" && self.line_name != ""
    }
}

#[derive(Debug, Clone)]
pub struct Vehicle {
    pub line: String,
    positions: Vec<walkers::Position>,
}

impl Vehicle {
    fn new(line: String) -> Self {
        Self {
            line,
            positions: Vec::new(),
        }
    }

    fn update(&mut self, position: walkers::Position) {
        self.positions.push(position);
        if self.positions.len() > 100 {
            self.positions.remove(0);
        }
    }

    /// Get the last position of the vehicle.
    pub fn position(&self) -> walkers::Position {
        self.positions.last().unwrap().clone()
    }
}

pub struct MpkWroclaw {
    #[allow(dead_code)]
    runtime: crate::io::Runtime,

    vehicles: Arc<Mutex<HashMap<String, Vehicle>>>,
}

impl MpkWroclaw {
    pub fn new(egui_ctx: egui::Context) -> Self {
        let vehicles = Arc::new(Mutex::new(HashMap::new()));
        let vehicles_clone = vehicles.clone();

        Self {
            vehicles,
            runtime: crate::io::Runtime::new(fetch_continuously(vehicles_clone, egui_ctx)),
        }
    }

    pub fn vehicles(&self) -> HashMap<String, Vehicle> {
        self.vehicles.lock().unwrap().clone()
    }
}

async fn fetch_continuously(
    vehicles: Arc<Mutex<HashMap<String, Vehicle>>>,
    egui_ctx: egui::Context,
) {
    loop {
        for position in &fetch_vehicles().await {
            vehicles
                .lock()
                .unwrap()
                .entry(position.id.clone())
                .or_insert_with(|| Vehicle::new(position.line_name.clone()))
                .update(walkers::lat_lon(position.latitude, position.longitude));
        }

        egui_ctx.request_repaint();
        sleep(Duration::from_secs(5)).await;
    }
}
