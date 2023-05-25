use std::{
	env,
	fs::OpenOptions,
	io::{Read, Write},
	process,
};

use anyhow::{anyhow, Result};
use chrono::Local;
use env_logger::{Builder, Target};
use log::LevelFilter;

use super::{
	config::Config,
	consts::CONFIG_FILE,
	path::{get_xdg_compat_dir, XDGDirType},
};

pub fn init_env_logger() -> Result<()> {
	let logs_dir = &get_xdg_compat_dir(XDGDirType::Cache)?.join("events.log");
	let target = OpenOptions::new()
		.create(true)
		.append(true)
		.open(logs_dir)?;
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
	let config_dir = get_xdg_compat_dir(XDGDirType::Config)?;
	let config_file = config_dir.join(CONFIG_FILE);
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
