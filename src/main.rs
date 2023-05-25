use std::{
	env,
	fs::{self, OpenOptions},
	io::{Read, Write},
	println,
	process::Command,
};

use anyhow::Result;
use clap::{command, Parser, Subcommand};
use shco::{
	config::Config,
	consts::CONFIG_LOCK,
	hash::get_config_hash,
	path::{get_xdg_compat_dir, XDGDirType},
	utils::{create_shell_init_script, get_rc_config, init_env_logger},
};
use sysinfo::{ProcessExt, Signal, System, SystemExt};
use url::Url;

mod shco;

#[derive(Debug, Subcommand)]
enum Commands {
	Init,
	Sync,
	Source,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CLIArgs {
	#[command(subcommand)]
	command: Commands,
}

fn main() -> Result<()> {
	init_env_logger()?;
	let CLIArgs { command } = CLIArgs::parse();

	match command {
		Commands::Init => {
			let shell_path = env::var("SHELL")?;
			let init_script = create_shell_init_script(&shell_path)?;
			println!("{}", &init_script);
		}
		Commands::Sync => {
			let config_hash = match get_config_hash() {
				Ok(hash) => hash,
				Err(_) => {
					log::error!("Could not get the current config's hash");
					return Ok(());
				}
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

			if config_hash != current_hash {
				let Config { plugins } = get_rc_config()?;

				let plugins_dir = &get_xdg_compat_dir(XDGDirType::Data)?.join("plugins");
				fs::create_dir_all(plugins_dir)?;

				for plugin in plugins {
					let plugin = plugin.as_str();
					log::debug!("Started working on {:?}", plugin);

					let plugin_url = Url::parse(&plugin)?;
					let plugin_parts = plugin_url
						.path_segments()
						.map(|seg_iter| seg_iter.rev().take(2).collect::<Vec<_>>());
					let Some(plugin_parts) = plugin_parts else { continue; };

					let (name, author) = match (plugin_parts.get(0), plugin_parts.get(1)) {
						(Some(name), Some(author)) => (name, author),
						_ => {
							log::warn!("`{}` is an invalid plugin URL", plugin);
							continue;
						}
					};
					let plug_git_dir = plugins_dir.join(author).join(name).join(".git");

					if !plug_git_dir.exists() {
						log::debug!("Installing '{}/{}'...", author, name);
						Command::new("git")
							.arg("clone")
							.arg(plugin)
							.current_dir(plugins_dir)
							.output()?;
						log::debug!("Installed '{}/{}' successfully", author, name);
					} else {
						log::debug!("Plugin '{}' already exists", plugin);
					}
					println!(
						include_str!("../assets/scripts/plugin_source.zsh"),
						plug_dir = plugins_dir.display(),
						author = author,
						plug_name = name
					);
				}
				config_lock_file.write_all(config_hash)?;

				if System::SUPPORTED_SIGNALS.contains(&Signal::Winch) {
					let mut sys = System::new();
					sys.refresh_processes();

					for process in sys.processes_by_name("zsh") {
						log::debug!(
							"Sending {} signal to pid #{} ({:?})",
							Signal::Winch,
							process.pid(),
							process.exe()
						);
						process.kill_with(Signal::Winch);
					}
				}
			} else {
				log::debug!("Locked hash and config's are the same, see you next time");
			}
		}
		Commands::Source => {
			let Config { plugins } = get_rc_config()?;
			let plugins_dir = &get_xdg_compat_dir(XDGDirType::Data)?.join("plugins");
			fs::create_dir_all(plugins_dir)?;

			for plugin in plugins {
				let plugin_url = Url::parse(&plugin)?;
				let plugin_parts = plugin_url
					.path_segments()
					.map(|seg_iter| seg_iter.rev().take(2).collect::<Vec<_>>());
				let Some(plugin_parts) = plugin_parts else { continue; };

				let (name, author) = match (plugin_parts.get(0), plugin_parts.get(1)) {
					(Some(name), Some(author)) => (name, author),
					_ => {
						log::warn!("`{}` is an invalid plugin URL", plugin);
						continue;
					}
				};

				if plugins_dir.join(author).join(name).join(".git").exists() {
					println!(
						include_str!("../assets/scripts/plugin_source.zsh"),
						plug_dir = plugins_dir.display().to_string(),
						author = author,
						plug_name = name
					);
				}
			}
		}
	}

	Ok(())
}
