use crate::ohos::{PLUGINS_TO_REGISTER, PLUGIN_MANAGER};
use crate::plugin::mobile::PENDING_PLUGIN_CALLS;
use napi_derive_ohos::napi;
use napi_ohos::bindgen_prelude::ObjectRef;
use napi_ohos::Env;

#[napi]
pub fn tauri_set_plugin_manager(_env: &Env, manager: ObjectRef) -> napi_ohos::Result<()> {
  PLUGIN_MANAGER.lock().unwrap().replace(manager);
  println!("[Tauri] PluginManager set from ArkTS");
  Ok(())
}

#[napi]
pub fn tauri_init_plugins(_env: &Env, manager: ObjectRef) -> napi_ohos::Result<String> {
  let plugins = PLUGINS_TO_REGISTER.lock().unwrap();
  let count = plugins.len();

  println!(
    "[Tauri] tauri_init_plugins called, {} plugins to register",
    count
  );

  PLUGIN_MANAGER.lock().unwrap().replace(manager);

  let plugins_json = serde_json::to_string(
    &plugins
      .iter()
      .map(|p| {
        serde_json::json!({
          "name": p.name,
          "identifier": p.identifier,
          "className": p.class_name,
          "config": p.config
        })
      })
      .collect::<Vec<_>>(),
  )
  .unwrap();

  println!("[Tauri] Plugins JSON: {}", plugins_json);

  Ok(plugins_json)
}

#[napi]
pub fn tauri_handle_plugin_response(id: i32, success: bool, payload: String) {
  let handler = PENDING_PLUGIN_CALLS
    .get_or_init(Default::default)
    .lock()
    .unwrap()
    .remove(&id);

  if let Some(handler) = handler {
    let json: serde_json::Value = serde_json::from_str(&payload).unwrap_or(serde_json::Value::Null);
    handler(if success { Ok(json) } else { Err(json) });
  }
}
