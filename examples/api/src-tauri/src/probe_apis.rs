//! S9 补漏探针命令：点亮 JS API 面未暴露、仅 Rust 侧可达的 App/Window 方法
//! （AppHandle monitor 四连 / app.rs+window/mod.rs set_menu+remove_menu /
//! Webview::reparent 的 "not supported on OHOS" 警告分支）。
//! 仅覆盖率插桩构建使用；语义与 driver 盲调用一致——执行即覆盖，错误聚合成字符串返回。

use tauri::Manager;

/// AppHandle 的 monitor/cursor 四连 + 每个 API 的返回摘要。
#[tauri::command]
pub fn probe_app_monitors<R: tauri::Runtime>(
  app: tauri::AppHandle<R>,
) -> Result<String, String> {
  let mut out = Vec::new();

  match app.primary_monitor() {
    Ok(Some(m)) => out.push(format!("primary={:?}", m.name())),
    Ok(None) => out.push("primary=None".to_string()),
    Err(e) => out.push(format!("primary=err({e})")),
  }

  match app.monitor_from_point(100.0, 200.0) {
    Ok(Some(m)) => out.push(format!("from_point={:?}", m.name())),
    Ok(None) => out.push("from_point=None".to_string()),
    Err(e) => out.push(format!("from_point=err({e})")),
  }

  match app.available_monitors() {
    Ok(monitors) => out.push(format!("available={}", monitors.len())),
    Err(e) => out.push(format!("available=err({e})")),
  }

  match app.cursor_position() {
    Ok(p) => out.push(format!("cursor={},{}", p.x, p.y)),
    Err(e) => out.push(format!("cursor=err({e})")),
  }

  Ok(out.join(" | "))
}

/// app.rs AppHandle::set_menu + remove_menu（app-wide 菜单装/卸）。
#[cfg(desktop)]
#[tauri::command]
pub fn probe_app_menu_set_remove<R: tauri::Runtime>(
  app: tauri::AppHandle<R>,
) -> Result<String, String> {
  let mut out = Vec::new();

  let menu = tauri::menu::Menu::new(&app).map_err(|e| e.to_string())?;
  match app.set_menu(menu) {
    Ok(prev) => out.push(format!("set_menu prev={:?}", prev.is_some())),
    Err(e) => out.push(format!("set_menu err({e})")),
  }

  match app.remove_menu() {
    Ok(prev) => out.push(format!("remove_menu prev={:?}", prev.is_some())),
    Err(e) => out.push(format!("remove_menu err({e})")),
  }

  Ok(out.join(" | "))
}

/// window/mod.rs Window::set_menu + remove_menu（窗口级菜单装/卸，含 OHOS menubar 分支）。
#[cfg(desktop)]
#[tauri::command]
pub fn probe_window_menu_set_remove<R: tauri::Runtime>(
  window: tauri::Window<R>,
) -> Result<String, String> {
  let mut out = Vec::new();

  let menu = tauri::menu::Menu::new(&window).map_err(|e| e.to_string())?;
  match window.set_menu(menu) {
    Ok(prev) => out.push(format!("set_menu prev={:?}", prev.is_some())),
    Err(e) => out.push(format!("set_menu err({e})")),
  }

  match window.remove_menu() {
    Ok(prev) => out.push(format!("remove_menu prev={:?}", prev.is_some())),
    Err(e) => out.push(format!("remove_menu err({e})")),
  }

  Ok(out.join(" | "))
}

/// Webview::reparent —— OHOS 上预期走 "not supported" 警告分支（覆盖目的即在此）。
#[tauri::command]
pub fn probe_webview_reparent<R: tauri::Runtime>(
  window: tauri::Window<R>,
) -> Result<String, String> {
  let webview = window
    .webviews()
    .into_iter()
    .next()
    .ok_or_else(|| "no webview on window".to_string())?;
  match webview.reparent(&window) {
    Ok(()) => Ok("reparent=ok".to_string()),
    Err(e) => Ok(format!("reparent=err({e})")),
  }
}
