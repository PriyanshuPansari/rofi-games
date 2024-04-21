use dirs::config_dir;
use lib_game_detector::data::{Game, GamesSlice};
use serde::Deserialize;
use std::{error::Error, fs::read_to_string};
use tracing::{debug, error};

use crate::utils::{get_launch_command, get_path_box_art, get_path_game_dir};

#[derive(Deserialize, Debug)]
pub struct Config {
    box_art_dir: Option<String>,
    entries: Vec<ConfigEntry>,
}

#[derive(Deserialize, Debug)]
pub struct ConfigEntry {
    title: String,
    launch_command: Option<String>,
    path_box_art: Option<String>,
    path_game_dir: Option<String>,
}

pub fn read_config() -> Option<Config> {
    let path_config = config_dir()?.join("rofi-games").join("config.toml");

    if !path_config.is_file() {
        debug!("Config file not found at {path_config:?}");
        return None;
    }

    debug!("Config file found at {:?}", &path_config);

    let Ok(contents) = read_to_string(path_config) else {
        error!("Could not read config file contents");
        return None;
    };

    toml::from_str::<Config>(&contents)
        .map_err(|e| {
            error!("Error parsing config: {:?}", e.message());
            if let Some(source) = e.source() {
                error!("Caused by: {source}");
            };
        })
        .ok()
}

pub fn add_custom_entries(entries: &GamesSlice, config: &Config) -> GamesSlice {
    // Convert parsed config entries into a `GamesSlice`
    let custom_entries: GamesSlice = config
        .entries
        .iter()
        .filter_map(|entry| {
            let ConfigEntry {
                title,
                launch_command: opt_launch_command,
                path_box_art: opt_path_box_art,
                path_game_dir: opt_path_game_dir,
            } = entry;

            let matching_entry = entries.iter().find(|e| &e.title == title);
            if matching_entry.is_none()
                && (opt_launch_command.is_none()
                    || opt_path_game_dir.is_none()
                    || opt_path_box_art.is_none())
            {
                error!("No matching entry found for the title '{title}'. All fields for the custom entry must be provided.");
                return None;
            };

            let launch_command = get_launch_command(matching_entry, opt_launch_command, title)?;
            let path_box_art = Some(get_path_box_art(matching_entry, opt_path_box_art, &config.box_art_dir, title)?);
            let path_game_dir = Some(get_path_game_dir(matching_entry, opt_path_game_dir,
                title)?);

            Some(Game {
                title: title.clone(),
                launch_command,
                path_box_art,
                path_game_dir,
            })
        })
        .collect();

    // Combine base entries with custom ones
    entries
        .iter()
        // Remove base entry if there is a custom entry to override it
        // NOTE: This can also remove multiple base entries per custom entry, since base entries
        // with the same title are allowed (from different sources). Not entirely sure how to avoid
        // this, so for now, it's a _feature_
        .filter(|g| custom_entries.iter().all(|c| c.title != g.title))
        .cloned()
        .chain(custom_entries.iter().cloned())
        .collect()
}
