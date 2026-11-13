//! Stable JSON protocol helpers for agent output.

use af_core::errors::AfError;
use serde::Serialize;
use serde_json::{json, Value};

pub const SCHEMA_VERSION: &str = "1";

#[derive(Serialize)]
pub struct ErrorEnvelope<'a> {
    pub ok: bool,
    #[serde(rename = "schemaVersion")]
    pub schema_version: &'a str,
    pub error: af_core::errors::ErrorPayload,
}

pub fn error_value(err: &AfError) -> Value {
    json!({
        "ok": false,
        "schemaVersion": SCHEMA_VERSION,
        "error": err.payload()
    })
}

pub fn success(mut v: Value) -> Value {
    if let Some(obj) = v.as_object_mut() {
        obj.entry("schemaVersion").or_insert_with(|| json!(SCHEMA_VERSION));
    }
    v
}
