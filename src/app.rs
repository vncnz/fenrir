// use std::fs;
use std::path::{PathBuf};
use std::error::Error;

#[derive(Debug, Clone)]
pub struct AppEntry {
    pub name: String,
    pub exec: String,
    pub icon_path: Option<PathBuf>,
    pub comment: String,
    pub terminal: bool
}

use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};

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