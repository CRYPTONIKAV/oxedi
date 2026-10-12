//! WebAssembly binding of the oxedi core for the browser viewer.
//!
//! Internal to the viewer: its JSON shape may change between releases.
//! `view` holds the picture of one file; this file only exports it to JS.

mod view;

pub use view::{
    ColumnView, Delimiters, DiagnosticView, NodeView, SegmentView, TableView, View, to_json, view,
};

use wasm_bindgen::prelude::*;

/// Parses an 835 and returns the viewer's JSON; throws with the core's
/// message when the bytes are not an X12 interchange.
#[wasm_bindgen]
pub fn parse(bytes: &[u8]) -> Result<String, JsError> {
    to_json(bytes).map_err(|message| JsError::new(&message))
}

/// The oxedi version this module was built from.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
