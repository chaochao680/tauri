// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use super::run_item_main_thread;
use crate::menu::CheckMenuItemInner;
use crate::run_main_thread;
use crate::{menu::MenuId, AppHandle, Manager, Runtime};

use super::CheckMenuItem;

impl<R: Runtime> CheckMenuItem<R> {
  /// Create a new menu item.
  ///
  /// - `text` could optionally contain an `&` before a character to assign this character as the mnemonic
  ///   for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
  pub fn new<M, T, A>(
    manager: &M,
    text: T,
    enabled: bool,
    checked: bool,
    accelerator: Option<A>,
  ) -> crate::Result<Self>
  where
    M: Manager<R>,
    T: AsRef<str>,
    A: AsRef<str>,
  {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let text = text.as_ref().to_owned();
    let accelerator = accelerator.and_then(|s| s.as_ref().parse().ok());

    #[cfg(target_env = "ohos")]
    let item = {
      let item = muda::CheckMenuItem::new(text, enabled, checked, accelerator);
      CheckMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    };

    #[cfg(not(target_env = "ohos"))]
    let item = run_main_thread!(handle, || {
      let item = muda::CheckMenuItem::new(text, enabled, checked, accelerator);
      CheckMenuItemInner {
        id: item.id().clone(),
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// Create a new menu item with the specified id.
  ///
  /// - `text` could optionally contain an `&` before a character to assign this character as the mnemonic
  ///   for this menu item. To display a `&` without assigning a mnemenonic, use `&&`.
  pub fn with_id<M, I, T, A>(
    manager: &M,
    id: I,
    text: T,
    enabled: bool,
    checked: bool,
    accelerator: Option<A>,
  ) -> crate::Result<Self>
  where
    M: Manager<R>,
    I: Into<MenuId>,
    T: AsRef<str>,
    A: AsRef<str>,
  {
    let handle = manager.app_handle();
    let app_handle = handle.clone();

    let id = id.into();
    let text = text.as_ref().to_owned();
    let accelerator = accelerator.and_then(|s| s.as_ref().parse().ok());

    #[cfg(target_env = "ohos")]
    let item = {
      let item = muda::CheckMenuItem::with_id(id.clone(), text, enabled, checked, accelerator);
      CheckMenuItemInner {
        id,
        inner: Some(item),
        app_handle,
      }
    };

    #[cfg(not(target_env = "ohos"))]
    let item = run_main_thread!(handle, || {
      let item = muda::CheckMenuItem::with_id(id.clone(), text, enabled, checked, accelerator);
      CheckMenuItemInner {
        id,
        inner: Some(item),
        app_handle,
      }
    })?;

    Ok(Self(Arc::new(item)))
  }

  /// The application handle associated with this type.
  pub fn app_handle(&self) -> &AppHandle<R> {
    &self.0.app_handle
  }

  /// Returns a unique identifier associated with this menu item.
  pub fn id(&self) -> &MenuId {
    &self.0.id
  }

  /// Get the text for this menu item.
  pub fn text(&self) -> crate::Result<String> {
    #[cfg(target_env = "ohos")]
    {
      Ok((*self.0).as_ref().text())
    }
    #[cfg(not(target_env = "ohos"))]
    {
      run_item_main_thread!(self, |self_: Self| (*self_.0).as_ref().text())
    }
  }

  /// Set the text for this check menu item.
  pub fn set_text<S: AsRef<str>>(&self, text: S) -> crate::Result<()> {
    let text = text.as_ref().to_string();
    #[cfg(target_env = "ohos")]
    {
      (*self.0).as_ref().set_text(text);
      super::auto_refresh_menubar(&self.0.app_handle);
      Ok(())
    }
    #[cfg(not(target_env = "ohos"))]
    {
      run_item_main_thread!(self, |self_: Self| (*self_.0).as_ref().set_text(text))
    }
  }

  /// Get whether this check menu item is enabled.
  pub fn is_enabled(&self) -> crate::Result<bool> {
    #[cfg(target_env = "ohos")]
    {
      Ok((*self.0).as_ref().is_enabled())
    }
    #[cfg(not(target_env = "ohos"))]
    {
      run_item_main_thread!(self, |self_: Self| (*self_.0).as_ref().is_enabled())
    }
  }

  /// Set whether this check menu item is enabled.
  pub fn set_enabled(&self, enabled: bool) -> crate::Result<()> {
    #[cfg(target_env = "ohos")]
    {
      (*self.0).as_ref().set_enabled(enabled);
      super::auto_refresh_menubar(&self.0.app_handle);
      Ok(())
    }
    #[cfg(not(target_env = "ohos"))]
    {
      run_item_main_thread!(self, |self_: Self| (*self_.0).as_ref().set_enabled(enabled))
    }
  }

  /// Set the accelerator for this check menu item.
  pub fn set_accelerator<S: AsRef<str>>(&self, accelerator: Option<S>) -> crate::Result<()> {
    let accel = accelerator.and_then(|s| s.as_ref().parse().ok());
    #[cfg(target_env = "ohos")]
    {
      let _ = (*self.0).as_ref().set_accelerator(accel);
      super::auto_refresh_menubar(&self.0.app_handle);
      Ok(())
    }
    #[cfg(not(target_env = "ohos"))]
    {
      run_item_main_thread!(self, |self_: Self| {
        (*self_.0).as_ref().set_accelerator(accel)
      })?
      .map_err(Into::into)
    }
  }

  /// Get whether this check menu item is checked.
  pub fn is_checked(&self) -> crate::Result<bool> {
    #[cfg(target_env = "ohos")]
    {
      Ok((*self.0).as_ref().is_checked())
    }
    #[cfg(not(target_env = "ohos"))]
    {
      run_item_main_thread!(self, |self_: Self| (*self_.0).as_ref().is_checked())
    }
  }

  /// Set whether this check menu item is checked.
  pub fn set_checked(&self, checked: bool) -> crate::Result<()> {
    #[cfg(target_env = "ohos")]
    {
      (*self.0).as_ref().set_checked(checked);
      super::auto_refresh_menubar(&self.0.app_handle);
      Ok(())
    }
    #[cfg(not(target_env = "ohos"))]
    {
      run_item_main_thread!(self, |self_: Self| (*self_.0).as_ref().set_checked(checked))
    }
  }
}
