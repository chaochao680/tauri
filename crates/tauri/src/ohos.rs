use std::sync::{Mutex, OnceLock};

pub use openharmony_ability;
pub use openharmony_ability_derive;
pub use tauri_runtime::OHOSWindowKind;

pub static APP: Mutex<Option<openharmony_ability::OpenHarmonyApp>> = Mutex::new(None);

/// Stores the base path for OHOS app, initialized before APP is taken.
pub static BASE_PATH: OnceLock<Option<String>> = OnceLock::new();

/// Stores the module name for OHOS app, initialized before APP is taken.
pub static MODULE_NAME: OnceLock<Option<String>> = OnceLock::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_path_and_module_name_accessors() {
        let _ = BASE_PATH.get();
        let _ = MODULE_NAME.get();
    }
}
