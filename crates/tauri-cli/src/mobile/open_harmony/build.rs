// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{
  delete_codegen_vars, ensure_init, env, get_app, get_config, inject_resources, log_finished,
  open_and_wait, plugins, signing::OhosSigningConfig, MobileTarget, OptionsHandle,
};
use crate::{
  build::Options as BuildOptions,
  helpers::{
    app_paths::{resolve_tauri_dir, Dirs},
    config::{get_config as get_tauri_config, ConfigMetadata},
    flock,
  },
  interface::{AppInterface, Options as InterfaceOptions},
  mobile::{write_options, CliOptions},
  ConfigValue, Result,
};
use clap::{ArgAction, Parser};

use crate::error::Context;
use cargo_mobile2::{
  open_harmony::{config::Config as OpenHarmonyConfig, env::Env, hap, target::Target},
  opts::{NoiseLevel, Profile},
  target::TargetTrait,
};


use std::collections::HashMap;
use std::env::{set_current_dir, set_var};
use std::ffi::OsString;

use std::path::Path;

#[derive(Debug, Clone, Parser)]
#[clap(
  about = "Build your app in release mode for OpenHarmony and generate HAPs",
  long_about = "Build your app in release mode for OpenHarmony and generate HAPs. It makes use of the `build.frontendDist` property from your `tauri.conf.json` file. It also runs your `build.beforeBuildCommand` which usually builds your frontend into `build.frontendDist`."
)]
pub struct Options {
  /// Builds with the debug flag
  #[clap(short, long)]
  pub debug: bool,
  /// Which targets to build (all by default).
  #[clap(
    short,
    long = "target",
    action = ArgAction::Append,
    num_args(0..),
    value_parser(clap::builder::PossibleValuesParser::new(Target::name_list()))
  )]
  pub targets: Option<Vec<String>>,
  /// List of cargo features to activate
  #[clap(short, long, action = ArgAction::Append, num_args(0..))]
  pub features: Option<Vec<String>>,
  /// JSON strings or paths to JSON, JSON5 or TOML files to merge with the default configuration file
  ///
  /// Configurations are merged in the order they are provided, which means a particular value overwrites previous values when a config key-value pair conflicts.
  ///
  /// Note that a platform-specific file is looked up and merged with the default file by default
  /// (tauri.macos.conf.json, tauri.linux.conf.json, tauri.windows.conf.json, tauri.android.conf.json, tauri.ios.conf.json and tauri.ohos.conf.json)
  /// but you can use this for more specific use cases such as different build flavors.
  #[clap(short, long)]
  pub config: Vec<ConfigValue>,
  /// Open DevEco Studio
  #[clap(short, long)]
  pub open: bool,
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  pub ci: bool,
  /// Device type to build for (mobile or desktop)
  #[clap(long, default_value = "mobile", value_parser(["mobile", "desktop"]))]
  pub device_type: String,
  /// Command line arguments passed to the runner.
  /// Use `--` to explicitly mark the start of the arguments.
  /// e.g. `tauri ohos build -- [runnerArgs]`.
  #[clap(last(true))]
  pub args: Vec<String>,
  /// Do not error out if a version mismatch is detected on a Tauri package.
  ///
  /// Only use this when you are sure the mismatch is incorrectly detected as version mismatched Tauri packages can lead to unknown behavior.
  #[clap(long)]
  pub ignore_version_mismatches: bool,
}

impl From<Options> for BuildOptions {
  fn from(options: Options) -> Self {
    Self {
      runner: None,
      debug: options.debug,
      target: None,
      features: options.features.unwrap_or_default(),
      bundles: None,
      no_bundle: false,
      config: options.config,
      args: options.args,
      ci: options.ci,
      skip_stapling: false,
      ignore_version_mismatches: options.ignore_version_mismatches,
      no_sign: false,
    }
  }
}

pub fn command(options: Options, noise_level: NoiseLevel) -> Result<()> {
  let dirs = crate::helpers::app_paths::resolve_dirs();

  // Set device type environment variable
  set_var("OHOS_DEVICE_TYPE", &options.device_type);

  delete_codegen_vars();

  let mut build_options: BuildOptions = options.clone().into();

  let first_target = Target::all()
    .get(
      options
        .targets
        .as_ref()
        .and_then(|l| l.first().map(|t| t.as_str()))
        .unwrap_or(Target::DEFAULT_KEY),
    )
    .unwrap();
  build_options.target = Some(first_target.triple.into());

  let tauri_config = get_tauri_config(
    tauri_utils::platform::Target::OpenHarmony,
    &options
      .config
      .iter()
      .map(|conf| &conf.0)
      .collect::<Vec<_>>(),
    dirs.tauri,
  )?;
  let (interface, config, metadata) = {
    let interface = AppInterface::new(&tauri_config, build_options.target.clone(), dirs.tauri)?;
    interface.build_options(&mut Vec::new(), &mut build_options.features, true);

    let app = get_app(MobileTarget::OpenHarmony, &tauri_config, &interface, dirs.tauri);

    let mut vars = HashMap::new();
    vars.insert("OHOS_DEVICE_TYPE".into(), OsString::from(&options.device_type));
    let cli_options = CliOptions {
      vars,
      ..Default::default()
    };

    let (config, metadata) = get_config(
      &app,
      &tauri_config,
      Some(&build_options.features),
      &cli_options,
    );
    (interface, config, metadata)
  };

  let profile = if options.debug {
    Profile::Debug
  } else {
    Profile::Release
  };

  let tauri_path = resolve_tauri_dir().unwrap();
  set_current_dir(tauri_path).with_context(|| "failed to change current working directory")?;

  ensure_init(
    &tauri_config,
    config.app(),
    config.project_dir(),
    MobileTarget::OpenHarmony,
    false
  )?;

  inject_plugins(&dirs.tauri, &config.project_dir())?;

  let mut env = env()?;

  crate::build::setup(&interface, &mut build_options, &tauri_config, &dirs, true)?;

  // run an initial build to initialize plugins
  first_target
    .build(&config, &metadata, &env, noise_level, true, profile)
    .context("failed to build OpenHarmony app")?;

  let open = options.open;
  let _handle = run_build(
    interface,
    options,
    build_options,
    tauri_config,
    profile,
    &config,
    &mut env,
    noise_level,
    dirs,
  )?;

  if open {
    open_and_wait(&config, &env);
  }

  Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_build(
  interface: AppInterface,
  options: Options,
  build_options: BuildOptions,
  tauri_config: ConfigMetadata,
  profile: Profile,
  config: &OpenHarmonyConfig,
  env: &mut Env,
  noise_level: NoiseLevel,
  dirs: Dirs,
) -> Result<OptionsHandle> {
  let interface_options = InterfaceOptions {
    debug: build_options.debug,
    target: build_options.target.clone(),
    args: build_options.args.clone(),
    ..Default::default()
  };

  let app_settings = interface.app_settings();
  let out_dir = app_settings.out_dir(&interface_options, dirs.tauri)?;
  let _lock = flock::open_rw(out_dir.join("lock").with_extension("ohos"), "OpenHarmony")?;

  let mut vars = HashMap::new();
  vars.insert(
    "OHOS_DEVICE_TYPE".into(),
    OsString::from(&options.device_type),
  );

  let cli_options = CliOptions {
    dev: false,
    features: build_options.features.clone(),
    args: build_options.args.clone(),
    noise_level,
    vars,
    config: build_options.config,
    target_device: None,
  };
  let handle = write_options(&tauri_config, cli_options)?;

  inject_resources(config, &tauri_config)?;
  super::inject_icons(config, &tauri_config, dirs.tauri)?;

  let hap_outputs = hap::build(config, env, noise_level, profile).context("failed to build hap")?;

  // Sign the HAP using hap-sign-tool.jar if environment variables are set
  let hap_outputs = sign_hap_if_configured(hap_outputs, env)?;

  log_finished(hap_outputs, "HAP");

  Ok(handle)
}

pub(crate) fn inject_plugins(
  tauri_dir: &Path,
  project_dir: &Path,
) -> Result<Vec<plugins::PluginMeta>> {
  log::info!("Starting OpenHarmony dynamic plugin injection");

  let detected_plugins =
    plugins::detect_all_plugins(tauri_dir).context("Plugin detection failed")?;

  if detected_plugins.is_empty() {
    log::info!("No OpenHarmony-compatible plugins detected, continuing build");
    return Ok(vec![]);
  }

  log::info!(
    "Detected {} OpenHarmony plugins: {:?}",
    detected_plugins.len(),
    detected_plugins.iter().map(|p| &p.name).collect::<Vec<_>>()
  );

  let metadata: Vec<plugins::PluginMeta> = detected_plugins
    .iter()
    .map(|d| plugins::parse_plugin_meta(&d.har_path, &d.name))
    .collect::<Result<Vec<_>>>()
    .context("Plugin metadata parsing failed")?;

  for plugin in &metadata {
    plugins::validate_plugin_meta(plugin)
      .context(format!("Invalid metadata for plugin '{}'", plugin.name))?;
  }

  for plugin in &metadata {
    plugins::copy_plugin_har(plugin, project_dir)
      .context(format!("Failed to copy plugin '{}' HAR", plugin.name))?;
  }

  plugins::update_plugin_configs(project_dir, &metadata)
    .context("Failed to update plugin configurations")?;

  plugins::validate_plugin_configs(project_dir, &metadata)
    .context("Plugin configuration validation failed")?;

  log::info!(
    "Build completed successfully with {} plugins",
    metadata.len()
  );
  Ok(metadata)
}

/// Sign HAP files if OHOS signing environment variables are configured.
///
/// For each unsigned HAP path, if a corresponding signed path exists alongside it,
/// the signed path is returned. Otherwise, the unsigned HAP is signed in-place
/// (output overwrites the unsigned path with a `-signed` suffix).
fn sign_hap_if_configured(
  hap_outputs: Vec<std::path::PathBuf>,
  env: &Env,
) -> Result<Vec<std::path::PathBuf>> {
  let signing_config = match OhosSigningConfig::from_env()? {
    Some(cfg) => cfg,
    None => {
      // No env vars set — check if any signed HAP already exists
      let has_signed = hap_outputs
        .iter()
        .any(|p| {
          let name = p.file_name().unwrap_or_default().to_string_lossy();
          name.contains("-signed")
        });
      if !has_signed {
        log::warn!(
          "No signed HAP found and OHOS signing environment variables are not set. \
           The HAP will not be installable on a device. \
           Set OHOS_KEYSTORE_FILE, OHOS_KEYSTORE_PASSWORD, OHOS_KEY_ALIAS, \
           OHOS_KEY_PASSWORD, OHOS_APP_CERT_FILE, and OHOS_PROFILE_FILE to enable signing."
        );
      }
      return Ok(hap_outputs);
    }
  };

  let mut signed_outputs = Vec::new();
  for hap_path in &hap_outputs {
    // Derive signed output path: entry-default-unsigned.hap -> entry-default-signed.hap
    let signed_path = hap_path
      .with_file_name(
        hap_path
          .file_name()
          .unwrap()
          .to_string_lossy()
          .replace("unsigned", "signed"),
      );

    signing_config
      .sign_hap(hap_path, &signed_path, env)
      .context("failed to sign HAP")?;

    signed_outputs.push(hap_path.clone());
    signed_outputs.push(signed_path);
  }

  Ok(signed_outputs)
}
