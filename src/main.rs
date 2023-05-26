use std::{
	env,
	fs::{self, OpenOptions},
	io::{Read, Write},
	println,
	process::Command,
	time::Instant,
};

use anyhow::Result;
use clap::{command, Parser, Subcommand};
use shco::{
	config::Config,
	consts::CONFIG_LOCK,
	hash::get_config_hash,
	path::{get_xdg_cache_home, get_xdg_data_home},
	utils::{create_shell_init_script, get_plugin_url_name_author, get_rc_config, init_env_logger},
};
use sysinfo::{ProcessExt, Signal, System, SystemExt};

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
			let now = Instant::now();

			let config_hash = match get_config_hash() {
				Ok(hash) => hash,
				Err(_) => {
					log::error!("Could not get the current config's hash");
					return Ok(());
				}
			};
			let config_hash = config_hash.as_bytes();

			let cache_dir = &get_xdg_cache_home()?;
			fs::create_dir_all(cache_dir)?;
			log::debug!("{:?} is available", cache_dir);

			let config_lock_file = &cache_dir.join(CONFIG_LOCK);
			let mut config_lock_file = OpenOptions::new()
				.read(true)
				.append(true)
				.create(true)
				.open(config_lock_file)?;

			let mut current_hash = vec![];
			config_lock_file.read_to_end(&mut current_hash)?;

			if config_hash != current_hash {
				let Config { plugins } = get_rc_config()?;

				let plugins_dir = &get_xdg_data_home()?.join("plugins");
				fs::create_dir_all(plugins_dir)?;

				for ref plugin in plugins {
					log::debug!("Started working on {:?}", plugin);

					let (ref name, ref author) = match get_plugin_url_name_author(plugin) {
						Ok((name, author)) => (name, author),
						Err(e) => {
							log::warn!("{}", e);
							continue;
						}
					};
					let plug_git_dir = plugins_dir.join(author).join(name).join(".git");

					if !plug_git_dir.exists() {
						log::debug!("Installing '{}/{}'...", author, name);
						let output = &Command::new("git")
							.arg("clone")
							.arg(plugin)
							.current_dir(plugins_dir)
							.output()?;
						log::debug!("Installed '{}/{}' successfully\n{:?}", author, name, output);
					} else {
						log::debug!("Plugin '{}' already exists", plugin);
					}

					log::debug!("Now sourcing {}/{}", author, name);
					println!(
						include_str!("../assets/scripts/plugin_source.zsh"),
						plug_dir = plugins_dir.display(),
						author = author,
						plug_name = name
					);
				}
				config_lock_file.write_all(config_hash)?;

				if let Some(shell_name) = &env::var("SHELL")?.split('/').last() {
					let mut sys = System::new();
					sys.refresh_processes();

					for process in sys.processes_by_name(shell_name) {
						match process.kill_with(Signal::Winch) {
							Some(true) => {
								log::debug!(
									"Sending `{}` signal to pid #{} ({:?})",
									Signal::Winch,
									process.pid(),
									process.exe()
								);
							}
							Some(false) => {
								log::debug!(
									"Could not send `{}` signal to pid #{} ({:?})",
									Signal::Winch,
									process.pid(),
									process.exe()
								);
							}
							None => {
								log::debug!(
									"The `{}` signal in not supported on this machine",
									Signal::Winch,
								);
							}
						}
					}
				}
			} else {
				log::debug!("Locked hash and config's are the same, see you next time");
			}

			log::trace!("Sync took {:?}", now.elapsed());
		}
		Commands::Source => {
			let Config { plugins } = get_rc_config()?;
			let plugins_dir = &get_xdg_data_home()?.join("plugins");
			fs::create_dir_all(plugins_dir)?;

			for ref plugin in plugins {
				let (ref name, ref author) = match get_plugin_url_name_author(plugin) {
					Ok((name, author)) => (name, author),
					Err(e) => {
						log::warn!("{}", e);
						continue;
					}
				};

				if plugins_dir.join(author).join(name).join(".git").exists() {
					log::debug!("Now sourcing {}/{}", author, name);
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
