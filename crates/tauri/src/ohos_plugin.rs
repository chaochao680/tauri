use crate::ohos::{PLUGINS_TO_REGISTER, PLUGIN_MANAGER, RUN_COMMAND_QUEUE, RUN_COMMAND_TSFN};
use crate::plugin::mobile::PENDING_PLUGIN_CALLS;
use napi_derive_ohos::napi;
use napi_ohos::bindgen_prelude::{FnArgs, Function, JsObjectValue, ObjectRef};
use napi_ohos::Env;

#[napi]
pub fn tauri_set_plugin_manager(_env: &Env, manager: ObjectRef) -> napi_ohos::Result<()> {
  PLUGIN_MANAGER.lock().unwrap().replace(manager);
  println!("[Tauri] PluginManager set from ArkTS");
  Ok(())
}

fn create_run_command_tsfn(env: &Env) -> napi_ohos::Result<()> {
  let callback: Function<'_, (), ()> =
    env.create_function_from_closure("run_command_callback", move |_ctx| {
      let env_rc = crate::ohos::openharmony_ability::get_main_thread_env();
      if let Some(env_ref) = env_rc.borrow().as_ref() {
        let manager_guard = PLUGIN_MANAGER.lock().unwrap();
        if let Some(manager_ref) = manager_guard.as_ref() {
          let manager_obj = manager_ref.get_value(env_ref)?;
          let run_command_fn = manager_obj.get_named_property::<Function<
            '_,
            FnArgs<(i32, String, String, String)>,
            (),
          >>("runCommand")?;

          let mut queue = RUN_COMMAND_QUEUE.lock().unwrap();
          while let Some(args) = queue.pop_front() {
            run_command_fn.call((args.id, args.plugin_name, args.command, args.payload).into())?;
          }
        }
      }
      Ok(())
    })?;

  let tsfn = callback
    .build_threadsafe_function()
    .callee_handled::<false>()
    .build()?;

  RUN_COMMAND_TSFN
    .get_or_init(|| std::sync::Mutex::new(None))
    .lock()
    .unwrap()
    .replace(tsfn);

  println!("[Tauri] run_command TSFN created");
  Ok(())
}

#[napi]
pub fn tauri_init_plugins(env: &Env, manager: ObjectRef) -> napi_ohos::Result<String> {
  let plugins = PLUGINS_TO_REGISTER.lock().unwrap();
  let count = plugins.len();

  println!(
    "[Tauri] tauri_init_plugins called, {} plugins to register",
    count
  );

  PLUGIN_MANAGER.lock().unwrap().replace(manager);

  create_run_command_tsfn(env)?;

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
