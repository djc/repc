use std::sync::Once;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use crate::embed;
use crate::embed::Rpc;
use crate::kv::idbstore::IdbStore;
use crate::kv::Store;

#[cfg(not(default))]
pub async fn new_idbstore(name: String) -> Option<Box<dyn Store>> {
    init_panic_hook();
    match IdbStore::new(&name).await {
        Ok(store) => Some(Box::new(store)),
        _ => None,
    }
}

#[wasm_bindgen]
pub async fn dispatch(db_name: String, rpc: u8, args: JsValue) -> Result<JsValue, JsValue> {
    init_panic_hook();
    let rpc = Rpc::from_u8(rpc).ok_or_else(|| JsValue::from(format!("Invalid RPC: {:?}", rpc)))?;
    embed::dispatch(db_name, rpc, args).await
}

static INIT: Once = Once::new();

pub fn init_console_log() {
    INIT.call_once(|| {
        if let Err(e) = console_log::init_with_level(log::Level::Info) {
            web_sys::console::error_1(&format!("Error registering console_log: {}", e).into());
        }
    });
}

fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    init_console_log();
}
