use std::path::PathBuf;

pub struct SshHostEntry {
    pub alias: String,
    pub hostname: String,
    pub identity_file: Option<String>,
    pub user: Option<String>,
}

fn ssh_config_path() -> Option<PathBuf> {
    let home = std::env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(dirs::home_dir)?;
    Some(home.join(".ssh").join("config"))
}

pub fn parse_ssh_config() -> Vec<SshHostEntry> {
    let path = match ssh_config_path() {
        Some(p) => p,
        None => return vec![],
    };

    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return vec![],
    };

    let mut entries: Vec<SshHostEntry> = Vec::new();
    let mut current: Option<SshHostEntry> = None;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (key, value) = match line.split_once(char::is_whitespace) {
            Some((k, v)) => (k.to_lowercase(), v.trim().to_string()),
            None => continue,
        };

        match key.as_str() {
            "host" => {
                if let Some(entry) = current.take() {
                    if !entry.hostname.is_empty() {
                        entries.push(entry);
                    }
                }
                if value != "*" {
                    current = Some(SshHostEntry {
                        alias: value,
                        hostname: String::new(),
                        identity_file: None,
                        user: None,
                    });
                }
            }
            "hostname" => {
                if let Some(ref mut entry) = current {
                    entry.hostname = value;
                }
            }
            "identityfile" => {
                if let Some(ref mut entry) = current {
                    entry.identity_file = Some(value);
                }
            }
            "user" => {
                if let Some(ref mut entry) = current {
                    entry.user = Some(value);
                }
            }
            _ => {}
        }
    }

    if let Some(entry) = current {
        if !entry.hostname.is_empty() {
            entries.push(entry);
        }
    }

    entries
}
