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
}
