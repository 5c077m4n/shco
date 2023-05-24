use std::{env, fs::OpenOptions, io::Read, process};

use anyhow::{anyhow, Result};

use super::{
	config::Config,
	consts::CONFIG_FILE,
	path::{get_xdg_compat_dir, XDGDirType},
};

pub fn print_shell_init(shell_path: &str) -> Result<()> {
	let bin_path = env::current_exe()?;
	let bin_path = bin_path.display().to_string() + " source";

	shell_path
		.split('/')
		.last()
		.ok_or_else(|| anyhow!("SHELL env var should not be empty"))
		.map(|shell| match shell {
			"zsh" => {
				println!(
					include_str!("../../assets/scripts/init.zsh"),
					cmd = bin_path
				);
			}
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
	Ok(serde_json::from_slice::<Config>(&config)?)
}
