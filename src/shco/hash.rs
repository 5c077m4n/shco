use anyhow::{bail, Result};
use sha256::try_digest;

use super::path::{get_xdg_compat_dir, XDGDirType};

pub fn get_config_hash() -> Result<String> {
	let config_file = get_xdg_compat_dir(XDGDirType::Config)?.join("rc.json");

	if config_file.try_exists()? {
		let config_file = config_file.as_path();
		let hash = try_digest(config_file)?;

		log::debug!(r#"Hash: "{}""#, &hash);
		Ok(hash)
	} else {
		bail!("Config file could not be read");
	}
}
