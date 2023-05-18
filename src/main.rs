use std::fs;

use anyhow::Result;
use shco::{
	hash::get_config_hash,
	path::{get_xdg_compat_dir, XDGDirType},
};

mod shco;

fn main() -> Result<()> {
	let config_hash = match get_config_hash() {
		Ok(hash) => hash,
		Err(_) => return Ok(()),
	};
	let config_hash = config_hash.as_str();

	let cache_dir = get_xdg_compat_dir(XDGDirType::CACHE)?;
	let cache_dir = cache_dir.as_path();
	fs::create_dir_all(cache_dir)?;

	let lock_file = &cache_dir.join("shco.lock");
	let current_hash = fs::read_to_string(lock_file)?;
	let current_hash = current_hash.as_str();

	if config_hash != current_hash {
		// TODO: handle config change
		fs::write(lock_file, config_hash)?;
	}

	Ok(())
}
