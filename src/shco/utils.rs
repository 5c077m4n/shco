use std::{env, process};

use anyhow::{anyhow, Result};

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
