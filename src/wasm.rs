use wasm_bindgen::prelude::*;

use crate::{
    get_save_version as polished_get_save_version, patch_save_bytes as polished_patch_save_bytes,
    patch_save_bytes_with_log as polished_patch_save_bytes_with_log, SaveBinary,
};

/// Returns the detected save version.
///
/// # Errors
/// Returns a JavaScript error string if the input buffer is not a supported save.
#[wasm_bindgen]
pub fn get_save_version(bytes: &[u8]) -> Result<u16, JsValue> {
    let save = SaveBinary::new(bytes.to_vec());
    polished_get_save_version(&save).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Applies either a fix patch (`dev_type != 0`) or a version migration (`dev_type == 0`).
///
/// # Errors
/// Returns a JavaScript error string if patching fails.
#[wasm_bindgen]
pub fn patch_save(bytes: &[u8], target_version: u16, dev_type: u8) -> Result<Vec<u8>, JsValue> {
    polished_patch_save_bytes(bytes.to_vec(), target_version, dev_type)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Applies a patch and returns a structured result object.
#[wasm_bindgen]
pub fn patch_save_with_log(bytes: &[u8], target_version: u16, dev_type: u8) -> JsValue {
    let outcome = polished_patch_save_bytes_with_log(bytes.to_vec(), target_version, dev_type);
    gb_save_web::js::patch_outcome_to_js(
        outcome.bytes.as_deref(),
        &outcome.logs,
        outcome.error.as_deref(),
    )
}
