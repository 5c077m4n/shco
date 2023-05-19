use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
	pub plugins: Vec<String>,
}
