use std::{
	env,
	fs::OpenOptions,
	io::{Read, Write},
	process,
};

use anyhow::{anyhow, bail, Result};
use chrono::Local;
use env_logger::{Builder, Target};
use log::LevelFilter;
use url::Url;

use super::{
	config::Config,
	consts::CONFIG_FILE,
	path::{get_xdg_cache_home, get_xdg_config_home},
};

pub fn init_env_logger() -> Result<()> {
	let log_file = &get_xdg_cache_home()?.join("events.log");
	let target = OpenOptions::new()
		.create(true)
		.append(true)
		.open(log_file)?;
	let target = Box::new(target);

	Builder::new()
		.format(|buf, record| {
			writeln!(
				buf,
				"{}:{} {} [{}] - {}",
				record.file().unwrap_or("unknown"),
				record.line().unwrap_or(0),
				Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"),
				record.level(),
				record.args()
			)
		})
		.target(Target::Pipe(target))
		.filter_level(LevelFilter::Trace)
		.init();

	Ok(())
}

pub fn create_shell_init_script(shell_path: &str) -> Result<String> {
	let cmd = {
		let bin = env::current_exe()?;
		bin.display().to_string()
	};
	shell_path
		.split('/')
		.last()
		.ok_or_else(|| anyhow!("SHELL env var should not be empty"))
		.map(|shell| match shell {
			"zsh" => format!(
				include_str!("../../assets/scripts/init.zsh"),
				cmd_source = cmd.clone() + " source",
				cmd_sync = cmd.clone() + " sync"
			),
			other => {
				log::error!("`{}` is an unsupported shell", &other);
				process::exit(1);
			}
		})
}

pub fn get_rc_config() -> Result<Config> {
	let config_file = &get_xdg_config_home()?.join(CONFIG_FILE);
	let mut config_file = OpenOptions::new()
		.read(true)
		.append(true)
		.create(true)
		.open(config_file)?;

	let mut config = vec![];
	config_file.read_to_end(&mut config)?;

	let config = serde_json::from_slice::<Config>(&config)?;
	Ok(config)
}

pub fn get_plugin_url_name_author(plug_url: &str) -> Result<(String, String)> {
	let plugin_url_object = Url::parse(plug_url)?;
	let plugin_parts = plugin_url_object
		.path_segments()
		.map(|seg_iter| seg_iter.rev().take(2).collect::<Vec<&str>>());

	if let Some(plugin_parts) = plugin_parts {
		match (plugin_parts.first(), plugin_parts.get(1)) {
			(Some(name), Some(author)) => Ok((name.to_string(), author.to_string())),
			_ => bail!("Could not extract the name and author of {}", plug_url),
		}
	} else {
		bail!("Could not extract the name and author of {}", plug_url);
	}
}
