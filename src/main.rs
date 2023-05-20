use std::{
	fs::{self, OpenOptions},
	io::{Read, Write},
	process::Command,
};

use anyhow::Result;
use shco::{
	config::Config,
	hash::get_config_hash,
	path::{get_xdg_compat_dir, XDGDirType},
};

mod shco;

const CONFIG_LOCK: &str = "config.lock";

fn main() -> Result<()> {
	env_logger::init();

	let config_hash = match get_config_hash() {
		Ok(hash) => hash,
		Err(_) => return Ok(()),
	};
	let config_hash = config_hash.as_bytes();

	let cache_dir = get_xdg_compat_dir(XDGDirType::Cache)?;
	let cache_dir = cache_dir.as_path();
	fs::create_dir_all(cache_dir)?;
	log::debug!("{:?} is available", cache_dir);

	let config_lock_file = cache_dir.join(CONFIG_LOCK);
	let config_lock_file = config_lock_file.as_path();
	let mut config_lock_file = OpenOptions::new()
		.read(true)
		.append(true)
		.create(true)
		.open(config_lock_file)?;

	let mut current_hash = vec![];
	config_lock_file.read_to_end(&mut current_hash)?;

	log::debug!("Current hash: {:?}", current_hash);

	if config_hash != current_hash {
		let config_dir = get_xdg_compat_dir(XDGDirType::Config)?;
		let config_file = config_dir.join("rc.json");
		let mut config_file = OpenOptions::new()
			.read(true)
			.append(true)
			.create(true)
			.open(config_file)?;

		let mut config = vec![];
		config_file.read_to_end(&mut config)?;
		let config: Config = serde_json::from_slice(&config)?;

		let plugins_dir = &get_xdg_compat_dir(XDGDirType::Data)?.join("plugins");
		fs::create_dir_all(plugins_dir)?;

		for plugin in config.plugins {
			let plugin = plugin.as_str();
			log::debug!("Started working on {:?}", plugin);

			let mut plugin_parts = plugin.split('/');
			let (name, author) = match (plugin_parts.next_back(), plugin_parts.next_back()) {
				(Some(name), Some(author)) => (name, author),
				_ => {
					log::warn!("[shco] `{}` is an invalid plugin URL", plugin);
					continue;
				}
			};
			let plug_git_dir = plugins_dir.join(author).join(name).join(".git");

			if !plug_git_dir.exists() {
				log::info!("[shco] Installing '{}/{}'...", author, name);
				Command::new("git")
					.arg("clone")
					.arg(plugin)
					.current_dir(plugins_dir)
					.output()?;
				log::info!("[shco] Installed '{}/{}' successfully", author, name);
			} else {
				log::debug!("Plugin '{}' already exists", plugin);
			}
		}
		config_lock_file.write(config_hash)?;
	} else {
		log::debug!("Locked hash and config's are the same, see you next time")
	}

	Ok(())
}
