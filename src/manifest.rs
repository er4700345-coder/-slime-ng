use std::fs;

#[derive(Debug, Clone)]
pub struct SlimeManifest {
    pub name: String,
    pub version: String,
    pub entry: String,
}

impl SlimeManifest {
    pub fn default() -> Self {
        Self {
            name: "slime-app".to_string(),
            version: "0.1.0".to_string(),
            entry: "main.slime".to_string(),
        }
    }

    pub fn load(path: &str) -> Self {
        let content = fs::read_to_string(path).unwrap_or_default();

        let mut manifest = Self::default();

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("name") {
                manifest.name = extract_value(line);
            } else if line.starts_with("version") {
                manifest.version = extract_value(line);
            } else if line.starts_with("entry") {
                manifest.entry = extract_value(line);
            }
        }

        manifest
    }
}

fn extract_value(line: &str) -> String {
    line.split('=')
        .nth(1)
        .unwrap_or("")
        .trim()
        .trim_matches('"')
        .to_string()
}
