use std::{env, path::PathBuf};

use anyhow::Result;

const LIB_NAME: &str = "shco";

pub enum XDGDirType {
	Cache,
	Config,
	Data,
}

pub fn get_xdg_compat_dir(dir_type: XDGDirType) -> Result<PathBuf> {
	let (dir_env, alt_dir) = match dir_type {
		XDGDirType::Cache => ("XDG_CACHE_HOME", ".cache"),
		XDGDirType::Config => ("XDG_CONFIG_HOME", ".config"),
		XDGDirType::Data => ("XDG_DATA_HOME", ".local/share"),
	};
	let path = if let Ok(cache_home) = env::var(dir_env) {
		let cache_home = cache_home.as_str();
		log::debug!("Env var {:?} is '{}'", dir_env, cache_home);

		PathBuf::from(cache_home).join(LIB_NAME)
	} else {
		log::debug!("{} not found, falling back", dir_env);

		let home_dir = env::var("HOME")?;
		PathBuf::from(home_dir).join(alt_dir).join(LIB_NAME)
	};

	Ok(path)
}
