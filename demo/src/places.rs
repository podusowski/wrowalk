//! Few common places in the city of Wrocław, used in the example app.

use walkers::{lon_lat, Position};

/// Main train station of the city of Wrocław.
/// https://en.wikipedia.org/wiki/Wroc%C5%82aw_G%C5%82%C3%B3wny_railway_station
pub fn wroclaw_glowny() -> Position {
    lon_lat(17.03664, 51.09916)
}
