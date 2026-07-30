// use std::fs;
use std::path::{PathBuf};
use std::error::Error;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AppEntry {
    pub name: String,
    pub exec: String,
    pub icon_path: Option<PathBuf>,
    pub comment: String,
    pub terminal: bool
}

use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LaunchStats {
    pub total_launches: u64,
    pub launches: Vec<String>,
}

pub type LaunchHistory = HashMap<String, LaunchStats>;

fn app_key(app: &AppEntry) -> String {
    format!("{}::{}", app.exec, app.name)
}

pub fn record_app_launch(app: &AppEntry, history: &mut LaunchHistory, at: DateTime<Utc>) {
    let key = app_key(app);
    let stats = history.entry(key).or_default();
    stats.total_launches += 1;
    stats.launches.push(at.to_rfc3339());
}

pub fn prune_launch_history(history: &mut LaunchHistory, max_age: Duration) {
    let cutoff = Utc::now() - max_age;
    for stats in history.values_mut() {
        stats.launches.retain(|ts| {
            DateTime::parse_from_rfc3339(ts).ok().map(|parsed| parsed.with_timezone(&Utc) >= cutoff).unwrap_or(false)
        });
        stats.total_launches = stats.launches.len() as u64;
    }
    history.retain(|_, stats| !stats.launches.is_empty());
}

pub fn sort_app_entries_by_launch_history(
    entries: &[AppEntry],
    history: &LaunchHistory,
    recent_window_days: i64,
) -> Vec<AppEntry> {
    let cutoff = Utc::now() - Duration::days(recent_window_days);
    let mut ranked: Vec<(f64, AppEntry)> = entries
        .iter()
        .cloned()
        .map(|entry| {
            let key = app_key(&entry);
            let stats = history.get(&key).cloned().unwrap_or_default();
            let recent_hits = stats
                .launches
                .iter()
                .filter(|ts| {
                    DateTime::parse_from_rfc3339(ts).ok().map(|parsed| parsed.with_timezone(&Utc) >= cutoff).unwrap_or(false)
                })
                .count() as f64;
            let score = recent_hits * 10.0 + stats.total_launches as f64;
            (score, entry)
        })
        .collect();

    ranked.sort_by(|left, right| {
        right.0.partial_cmp(&left.0).unwrap_or(std::cmp::Ordering::Equal)
    });

    ranked.into_iter().map(|(_, entry)| entry).collect()
}

pub fn save_launch_history(history: &LaunchHistory, path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let data = serde_json::to_string_pretty(history)?;
    fs::write(path, data)?;
    Ok(())
}

pub fn load_launch_history(path: &PathBuf) -> Result<LaunchHistory, Box<dyn Error>> {
    if !path.exists() {
        return Ok(LaunchHistory::new());
    }
    let data = fs::read_to_string(path)?;
    if data.trim().is_empty() {
        return Ok(LaunchHistory::new());
    }
    Ok(serde_json::from_str(&data)?)
}

pub fn load_app_entries() -> Result<Vec<AppEntry>, Box<dyn Error>> {

    let mut results = vec![];
    let locales = get_languages_from_env();

    let entries = Iter::new(default_paths())
        .entries(Some(&locales))
        .collect::<Vec<_>>();
    
    for entry in entries {
        // let path_src = PathSource::guess_from(&entry.path);
        // println!("{:?}: {}\n---\n{}", path_src, entry.path.display(), entry);
        if (&entry).no_display() == false {
            results.push(AppEntry {
                exec: (&entry).exec().unwrap_or_default().to_string(),
                name: (&entry).name(&vec!["en"]).as_ref().unwrap().to_string(),
                icon_path: resolve_icon_path((&entry).icon().unwrap_or_default().to_string()),
                comment: (&entry).comment(&vec!["en"]).unwrap_or_default().to_string(),
                terminal: (&entry).terminal()
            });
        }
    }

    Ok(results)
}



/*
pub fn load_app_entries_OLD() -> Result<Vec<AppEntry>, Box<dyn Error>> {
    let mut entries = vec![];
    let paths = fs::read_dir("/usr/share/applications")?;
    
    for entry in paths {
        let path = entry?.path();
        if path.extension().map(|ext| ext == "desktop").unwrap_or(false) {
            let contents = fs::read_to_string(&path)?;
            let name = extract_field(&contents, "Name").unwrap_or_default();
            let exec = extract_field(&contents, "Exec").unwrap_or_default();
            let icon = extract_field(&contents, "Icon");
            let comment = extract_field(&contents, "Comment").unwrap_or_default();

            let icon_path = icon.and_then(resolve_icon_path);

            entries.push(AppEntry {
                name,
                exec,
                icon_path,
                comment,
                terminal: false
            });
        }
    }
    Ok(entries)
}
*/
/*
fn extract_field(contents: &str, field: &str) -> Option<String> {
    contents
        .lines()
        .find(|line| line.starts_with(&format!("{}=", field)))
        .map(|line| line.split_once('=').unwrap().1.trim().to_string())
}
*/

fn resolve_icon_path(icon_name: String) -> Option<PathBuf> {
    let candidates = vec![
        format!("/usr/share/icons/hicolor/48x48/apps/{}.png", icon_name),
        format!("/usr/share/pixmaps/{}.png", icon_name)
    ];

    candidates.into_iter().map(PathBuf::from).find(|p| p.exists())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn ranks_recently_used_apps_above_older_ones() {
        let now = Utc::now();
        let recent = AppEntry {
            name: "Recent".into(),
            exec: "recent".into(),
            icon_path: None,
            comment: "".into(),
            terminal: false,
        };
        let old = AppEntry {
            name: "Old".into(),
            exec: "old".into(),
            icon_path: None,
            comment: "".into(),
            terminal: false,
        };

        let mut history = HashMap::new();
        record_app_launch(&recent, &mut history, now - Duration::days(2));
        record_app_launch(&recent, &mut history, now - Duration::days(10));
        record_app_launch(&old, &mut history, now - Duration::days(60));

        let ranked = sort_app_entries_by_launch_history(&[old.clone(), recent.clone()][..], &history, 30);

        assert_eq!(ranked[0].name, recent.name);
        assert_eq!(ranked[1].name, old.name);
    }

    #[test]
    fn records_launches_in_history() {
        let app = AppEntry {
            name: "Test".into(),
            exec: "test".into(),
            icon_path: None,
            comment: "".into(),
            terminal: false,
        };

        let mut history = HashMap::new();
        record_app_launch(&app, &mut history, Utc::now());

        let stats = history.get(&app_key(&app)).unwrap();
        assert_eq!(stats.total_launches, 1);
        assert_eq!(stats.launches.len(), 1);
    }
}