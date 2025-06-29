use serde::Deserialize;
use std::collections::HashMap;

pub async fn fetch_positions() -> Vec<MpkPosition> {
    let url =
        "https://www.wroclaw.pl/open-data/datastore/dump/a9b3841d-e977-474e-9e86-8789e470a85a";
    let result = reqwest::get(url).await.unwrap();
    let bytes = result.bytes().await.unwrap();

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(bytes.as_ref());

    rdr.deserialize()
        .map(|record| {
            let record: MpkPosition = record.unwrap();
            record
        })
        .filter(MpkPosition::sane)
        .collect()
}

#[derive(Deserialize, Debug)]
pub struct MpkPosition {
    #[serde(rename = "_id")]
    id: String,
    #[serde(rename = "Nr_Boczny")]
    fleet_number: String,
    #[serde(rename = "Nr_Rej")]
    registration_number: String,
    #[serde(rename = "Brygada")]
    brigade: String,
    #[serde(rename = "Nazwa_Linii")]
    line_name: String,
    #[serde(rename = "Ostatnia_Pozycja_Szerokosc")]
    latitude: f64,
    #[serde(rename = "Ostatnia_Pozycja_Dlugosc")]
    longitude: f64,
    #[serde(rename = "Data_Aktualizacji")]
    last_update: String,
}

impl MpkPosition {
    /// Does this record even make sense.
    fn sane(position: &MpkPosition) -> bool {
        position.line_name != "None" && position.line_name != ""
    }
}

impl From<MpkPosition> for walkers::extras::LabeledSymbol {
    fn from(position: MpkPosition) -> Self {
        walkers::extras::LabeledSymbol {
            position: walkers::lat_lon(position.latitude, position.longitude),
            label: format!("{}", position.line_name),
            symbol: '🚌',
            style: walkers::extras::LabeledSymbolStyle::default(),
        }
    }
}
