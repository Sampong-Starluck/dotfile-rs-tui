use nanoserde::DeJson;
use crate::models::{AppEntry, AppSection, Apps};

pub fn read_apps_json() -> Result<Apps, Box<dyn std::error::Error>> {
    const APPS_JSON: &str = include_str!("../json/apps.json");
    let apps: Apps = DeJson::deserialize_json(APPS_JSON)
        .map_err(|e| -> Box<dyn std::error::Error> { e.to_string().into() })?;
    Ok(apps)
}

pub fn filter_apps_by_platform(apps: &Apps, detected_mgr: &str) -> Apps {
    let os = if cfg!(target_os = "windows") { "windows" }
    else if cfg!(target_os = "macos") { "macos" }
    else { "linux" };

    apps.iter()
        .filter_map(|section| {
            let filtered_apps: Vec<AppEntry> = section.apps
                .iter()
                .filter(|entry| {
                    let Some(platform) = entry.platforms.get(os) else {
                        return false;
                    };
                    platform.contains_key(detected_mgr)
                })
                .cloned()
                .collect();

            if filtered_apps.is_empty() {
                None
            } else {
                Some(AppSection {
                    section: section.section.clone(),
                    apps: filtered_apps,
                })
            }
        })
        .collect()
}