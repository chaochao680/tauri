use std::sync::{Mutex, OnceLock};

#[cfg(target_env = "ohos")]
use napi_ohos::bindgen_prelude::ObjectRef;

pub use openharmony_ability;
pub use openharmony_ability_derive;

pub static APP: Mutex<Option<openharmony_ability::OpenHarmonyApp>> = Mutex::new(None);

/// Stores the base path for OHOS app, initialized before APP is taken.
pub static BASE_PATH: OnceLock<Option<String>> = OnceLock::new();

/// Stores the module name for OHOS app, initialized before APP is taken.
pub static MODULE_NAME: OnceLock<Option<String>> = OnceLock::new();

/// Stores the PluginManager instance for OHOS, used to run plugin commands.
#[cfg(target_env = "ohos")]
pub static PLUGIN_MANAGER: Mutex<Option<ObjectRef>> = Mutex::new(None);

/// Stores plugins to be registered during tauriInitPlugins
#[cfg(target_env = "ohos")]
pub struct PluginRegistration {
  pub name: String,
  pub identifier: String,
  pub class_name: String,
  pub config: String,
}

#[cfg(target_env = "ohos")]
pub static PLUGINS_TO_REGISTER: Mutex<Vec<PluginRegistration>> = Mutex::new(Vec::new());
