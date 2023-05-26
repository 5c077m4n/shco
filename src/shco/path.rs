use std::{env, path::PathBuf};

use anyhow::Result;

const LIB_NAME: &str = "shco";

enum XDGDirType {
	Cache,
	Config,
	Data,
}
fn get_xdg_compat_dir(dir_type: XDGDirType) -> Result<PathBuf> {
	let (dir_env, alt_dir) = match dir_type {
		XDGDirType::Cache => ("XDG_CACHE_HOME", ".cache"),
		XDGDirType::Config => ("XDG_CONFIG_HOME", ".config"),
		XDGDirType::Data => ("XDG_DATA_HOME", ".local/share"),
	};
	env::var(dir_env)
		.map_or_else(
			|_err| {
				let home_dir = env::var("HOME")?;
				let path = PathBuf::from(home_dir).join(alt_dir);
				Ok(path)
			},
			|xdg_home| {
				let path = PathBuf::from(xdg_home.as_str());
				Ok(path)
			},
		)
		.map(|path| path.join(LIB_NAME))
}

pub fn get_xdg_cache_home() -> Result<PathBuf> {
	get_xdg_compat_dir(XDGDirType::Cache)
}
pub fn get_xdg_config_home() -> Result<PathBuf> {
	get_xdg_compat_dir(XDGDirType::Config)
}
pub fn get_xdg_data_home() -> Result<PathBuf> {
	get_xdg_compat_dir(XDGDirType::Data)
}
