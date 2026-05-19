// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::error::Context;
use crate::helpers::template;
use crate::mobile::open_harmony::plugins::{validate_plugin_meta, PluginMeta};
use crate::Result;
use cargo_mobile2::{
  config::app::App,
  open_harmony::{config::Config, target::Target},
  os,
  target::TargetTrait,
  util,
};
use handlebars::Handlebars;
use include_dir::{include_dir, Dir};
use serde_json::json;

use std::path::Path;

const TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/mobile/open-harmony");

pub fn gen(
  app: &App,
  config: &Config,
  (handlebars, mut map): (Handlebars, template::JsonMap),
  skip_targets_install: bool,
) -> Result<()> {
  gen_with_plugins(app, config, (handlebars, map), skip_targets_install, vec![])
}

pub fn gen_with_plugins(
  app: &App,
  config: &Config,
  (handlebars, mut map): (Handlebars, template::JsonMap),
  skip_targets_install: bool,
  plugins: Vec<PluginMeta>,
) -> Result<()> {
  if !skip_targets_install {
    let installed_targets =
      crate::interface::rust::installation::installed_targets().unwrap_or_default();
    let missing_targets = Target::all()
      .values()
      .filter(|t| !installed_targets.contains(&t.triple().into()))
      .collect::<Vec<&Target>>();

    if !missing_targets.is_empty() {
      println!("Installing OpenHarmony Rust toolchains...");
      for target in missing_targets {
        target
          .install()
          .context("failed to install target with rustup")?;
      }
    }
  }

  println!("Generating DevEco Studio project...");
  let dest = config.project_dir();

  map.insert(
    "root-dir-rel",
    Path::new(&os::replace_path_separator(
      util::relativize_path(app.root_dir(), &dest.join("entry")).into_os_string(),
    )),
  );
  map.insert("root-dir", app.root_dir());
  map.insert("windows", cfg!(windows));

  populate_template(handlebars, map, plugins, &dest)?;

  Ok(())
}

pub fn populate_template(
  handlebars: Handlebars,
  map: template::JsonMap,
  plugins: Vec<PluginMeta>,
  dest: &Path,
) -> Result<()> {
  log::info!("Populating template with {} plugins", plugins.len());

  for plugin in &plugins {
    validate_plugin_meta(plugin)
      .context(format!("Invalid plugin metadata for '{}'", plugin.name))?;
  }

  let plugin_data = plugins
    .iter()
    .map(|p| {
      json!({
        "name": p.name,
        "identifier": p.identifier,
        "className": p.class_name,
      })
    })
    .collect::<Vec<_>>();

  let mut data = map.inner().clone();
  data.insert("plugins".to_string(), json!(plugin_data));

  template::render(&handlebars, &data, &TEMPLATE_DIR, dest)
    .with_context(|| "failed to process template")?;

  log::info!("Template populated successfully");
  Ok(())
}
