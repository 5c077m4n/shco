use std::{fs, println, process::Command};

use anyhow::Result;
use shco::{
	config::Config,
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

	let cache_dir = get_xdg_compat_dir(XDGDirType::Cache)?;
	let cache_dir = cache_dir.as_path();
	fs::create_dir_all(cache_dir)?;

	let lock_file = &cache_dir.join("shco.lock");
	let current_hash = fs::read_to_string(lock_file)?;
	let current_hash = current_hash.as_str();

	if config_hash != current_hash {
		let config_dir = get_xdg_compat_dir(XDGDirType::Config)?;
		let config_file = config_dir.join("rc.json");

		let config = &fs::read_to_string(config_file)?;
		let config: Config = serde_json::from_str(config)?;

		let plugins_dir = get_xdg_compat_dir(XDGDirType::Data)?.join("plugins");
		for plugin in config.plugins {
			let plugin = plugin.as_str();

			let mut plugin_parts = plugin.split('/');
			let (name, author) = match (plugin_parts.next_back(), plugin_parts.next_back()) {
				(Some(name), Some(author)) => (name, author),
				_ => {
					eprintln!("[shco] `{}` is an invalid plugin URL", plugin);
					continue;
				}
			};
			let plug_local_dir = plugins_dir.join(author).join(name);

			if plug_local_dir.exists() {
				let mut git = Command::new("git pull origin/main");
				git.current_dir(plug_local_dir);

				println!("Updating {}/{}...", author, name);
				git.spawn()?;
			} else {
				let mut git = Command::new(format!("git clone {}", plugin));
				git.current_dir(plug_local_dir);

				println!("Installing {}/{}...", author, name);
				git.spawn()?;
			}
		}
		fs::write(lock_file, config_hash)?;
	}

	Ok(())
}
