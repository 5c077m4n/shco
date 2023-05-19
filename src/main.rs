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

	let lock_file = cache_dir.join("shco.lock");
	let lock_file = lock_file.as_path();
	let mut lock_file = OpenOptions::new()
		.read(true)
		.append(true)
		.create(true)
		.open(lock_file)?;

	let mut current_hash = vec![];
	lock_file.read_to_end(&mut current_hash)?;

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

		let plugins_dir = get_xdg_compat_dir(XDGDirType::Data)?.join("plugins");
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
			let plug_local_dir = &plugins_dir.join(author).join(name);
			fs::create_dir_all(plug_local_dir)?;

			if plug_local_dir.exists() {
				let mut git = Command::new("git pull origin/main");
				git.current_dir(plug_local_dir);

				log::info!("Updating {}/{}...", author, name);
				git.spawn()?;
			} else {
				let mut git = Command::new(format!("git clone {}", plugin));
				git.current_dir(plug_local_dir);

				log::info!("Installing {}/{}...", author, name);
				git.spawn()?;
			}
		}
		lock_file.write(config_hash)?;
	}

	Ok(())
}
