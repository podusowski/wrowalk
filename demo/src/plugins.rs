use walkers::{
    extras::{LabeledSymbol, LabeledSymbolStyle, Places},
    Plugin,
};

use crate::places;

/// Creates a built-in [`GroupedPlaces`] plugin populated with some predefined places.
pub fn places() -> impl Plugin {
    Places::new(vec![
        LabeledSymbol {
            position: places::wroclaw_glowny(),
            label: "Wrocław Główny\ntrain station".to_owned(),
            symbol: '🚆',
            style: LabeledSymbolStyle::default(),
        },
        LabeledSymbol {
            position: places::dworcowa_bus_stop(),
            label: "Bus stop".to_owned(),
            symbol: '🚌',
            style: LabeledSymbolStyle::default(),
        },
    ])
}
