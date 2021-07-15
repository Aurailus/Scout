use serde::{ Serialize, Deserialize };
use std::io::{ Read, Write, Seek, SeekFrom };

use scout_core::Shared;

fn default_opacity() -> u32 { 90 }

fn default_hide_on_unfocus() -> bool { true }

fn default_always_on_top() -> bool { true }

fn default_developer() -> bool { false }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Preferences {

	#[serde(skip_serializing, default)]
	pub path: std::path::PathBuf,

	#[serde(default = "default_opacity")]
	pub opacity: u32,

	#[serde(default = "default_hide_on_unfocus")]
	pub hide_on_unfocus: bool,

	#[serde(default = "default_always_on_top")]
	pub always_on_top: bool,

	#[serde(default = "default_developer")]
	pub developer: bool
}

impl Preferences {
	pub fn new(path: Option<&std::path::Path>) -> Shared<Self> {
		let path = path
			.and_then(|path| Some(path.to_owned()))
			.unwrap_or_else(|| std::path::Path::new(&[ "/home/", &whoami::username(), "/.config/scout.conf" ].join(""))
			.to_owned());

		let mut file = std::fs::OpenOptions::new().read(true).write(true).create(true).open(&path).unwrap();
		let mut contents = String::new();
		drop(file.read_to_string(&mut contents));
		drop(file);

		let prefs = Shared::new(match serde_json::from_str::<Preferences>(&contents) {
			Ok(json) => json,
			Err(err) => {
				println!("Error reading config file, resetting to default. {:?}", err);
				serde_json::from_str("{}").unwrap()
			}
		});

		let mut prefs_mut = prefs.borrow_mut();
		prefs_mut.path = path;
		prefs_mut.save().unwrap();
		drop(prefs_mut);

		prefs
	}

	pub fn save(&self) -> std::io::Result<()> {
		let mut file = std::fs::OpenOptions::new().read(true).write(true).create(true).open(&self.path)?;

		file.set_len(0)?;
		file.seek(SeekFrom::Start(0))?;
		file.write_all(serde_json::to_string(&self).unwrap().as_bytes())?;

		Ok(())
	}
}
