use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

async fn fetch_vehicles() -> Vec<RawVehicleRecord> {
    let url =
        "https://www.wroclaw.pl/open-data/datastore/dump/a9b3841d-e977-474e-9e86-8789e470a85a";
    let result = reqwest::get(url).await.unwrap();
    let bytes = result.bytes().await.unwrap();

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(bytes.as_ref());

    rdr.deserialize()
        .map(|record| {
            let record: RawVehicleRecord = record.unwrap();
            record
        })
        .filter(RawVehicleRecord::sane)
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
    fn sane(position: &RawVehicleRecord) -> bool {
        position.line_name != "None" && position.line_name != ""
    }
}

#[derive(Debug, Clone)]
pub struct Vehicle {
    pub line: String,
    positions: Vec<walkers::Position>,
}

impl Vehicle {
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
            runtime: crate::io::Runtime::new(async move {
                loop {
                    let positions = fetch_vehicles().await;
                    log::trace!("Fetched positions: {:#?}", positions);

                    {
                        let mut vehicles_lock = vehicles_clone.lock().unwrap();
                        for position in &positions {
                            vehicles_lock
                                .entry(position.id.clone())
                                .or_insert_with(|| Vehicle {
                                    line: position.line_name.clone(),
                                    positions: Vec::new(),
                                })
                                .update(walkers::lat_lon(position.latitude, position.longitude));
                        }

                        log::debug!("Vehicles: {:#?}", vehicles_lock);
                    }

                    egui_ctx.request_repaint();
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }),
        }
    }

    pub fn vehicles(&self) -> HashMap<String, Vehicle> {
        self.vehicles.lock().unwrap().clone()
    }
}
