use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

#[cfg(not(target_arch = "wasm32"))]
use tokio::time::sleep;

#[cfg(target_arch = "wasm32")]
use wasmtimer::tokio::sleep;

use walkers::Position;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

static APP_IN_BACKGROUND: OnceLock<AtomicBool> = OnceLock::new();

#[no_mangle]
pub extern "C" fn Java_com_github_podusowski_wrowalk_MainActivity_setAppInBackground(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    is_background: bool,
) {
    let atomic = APP_IN_BACKGROUND.get_or_init(|| AtomicBool::new(false));
    atomic.store(is_background, Ordering::SeqCst);
}

pub fn is_app_in_background() -> bool {
    APP_IN_BACKGROUND
        .get()
        .map(|a| a.load(Ordering::SeqCst))
        .unwrap_or(false)
}

async fn fetch_vehicles() -> Vec<RawVehicleRecord> {
    log::info!("Fetching vehicles from Wroclaw Open Data.");

    let url =
        "https://www.wroclaw.pl/open-data/datastore/dump/a9b3841d-e977-474e-9e86-8789e470a85a";
    let bytes = reqwest::get(url).await.unwrap().bytes().await.unwrap();

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
        self.line_name != "None"
            && !self.line_name.is_empty()
            && (self.longitude - 16.0).abs() < 10.0
            && (self.latitude - 52.0).abs() < 10.0
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
        if self.positions.last() != Some(&position) {
            self.positions.push(position);
        }
        if self.positions.len() > 10 {
            self.positions.remove(0);
        }
    }

    /// Get the last position of the vehicle.
    pub fn position(&self) -> walkers::Position {
        *self.positions.last().unwrap()
    }

    pub fn positions(&self) -> Vec<Position> {
        self.positions.clone()
    }
}

pub struct MpkWroclaw {
    #[allow(dead_code)]
    runtime: crate::io::Runtime,

    vehicles: Arc<Mutex<HashMap<String, Vehicle>>>,
}

/// Tracks vehicles in Wroclaw and keeps a short history.
impl MpkWroclaw {
    pub fn new(egui_ctx: egui::Context) -> Self {
        let vehicles = Arc::new(Mutex::new(HashMap::new()));

        Self {
            vehicles: vehicles.clone(),
            runtime: crate::io::Runtime::new(fetch_continuously(vehicles, egui_ctx)),
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
        if !is_app_in_background() {
            for position in &fetch_vehicles().await {
                vehicles
                    .lock()
                    .unwrap()
                    .entry(position.id.clone())
                    .or_insert_with(|| Vehicle::new(position.line_name.clone()))
                    .update(walkers::lat_lon(position.latitude, position.longitude));
            }

            log::debug!("Vehicles: {:#?}", vehicles.lock().unwrap());

            egui_ctx.request_repaint();
        } else {
            log::info!("App is in background, skipping fetch.");
            vehicles.lock().unwrap().clear();
        }

        sleep(Duration::from_secs(5)).await;
    }
}
