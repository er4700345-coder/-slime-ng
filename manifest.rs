// src/pkg/manifest.rs
// SLIME Package Manifest — slime.toml parser and types

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// slime.toml — the package manifest file
#[derive(Debug, Clone)]
pub struct Manifest {
    pub package: PackageMeta,
    pub dependencies: HashMap<String, VersionReq>,
    pub dev_dependencies: HashMap<String, VersionReq>,
}

#[derive(Debug, Clone)]
pub struct PackageMeta {
    pub name: String,
    pub version: Version,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub entry: String, // default: "src/main.slime"
}

/// Semantic version: major.minor.patch
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.trim().split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid version: {}", s));
        }
        Ok(Version {
            major: parts[0].parse().map_err(|_| format!("Bad major in {}", s))?,
            minor: parts[1].parse().map_err(|_| format!("Bad minor in {}", s))?,
            patch: parts[2].parse().map_err(|_| format!("Bad patch in {}", s))?,
        })
    }

    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Version requirement: ^1.2.3, ~1.2.3, =1.2.3, *
#[derive(Debug, Clone)]
pub enum VersionReq {
    Caret(Version),   // ^1.2.3 — compatible (same major)
    Tilde(Version),   // ~1.2.3 — patch updates only
    Exact(Version),   // =1.2.3 — exact match
    Any,              // *
}

impl VersionReq {
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim();
        if s == "*" {
            return Ok(VersionReq::Any);
        }
        if let Some(v) = s.strip_prefix('^') {
            return Ok(VersionReq::Caret(Version::parse(v)?));
        }
        if let Some(v) = s.strip_prefix('~') {
            return Ok(VersionReq::Tilde(Version::parse(v)?));
        }
        if let Some(v) = s.strip_prefix('=') {
            return Ok(VersionReq::Exact(Version::parse(v)?));
        }
        // bare version = caret by default
        Ok(VersionReq::Caret(Version::parse(s)?))
    }

    pub fn matches(&self, v: &Version) -> bool {
        match self {
            VersionReq::Any => true,
            VersionReq::Exact(req) => v == req,
            VersionReq::Caret(req) => {
                v.major == req.major && *v >= *req
            }
            VersionReq::Tilde(req) => {
                v.major == req.major && v.minor == req.minor && v.patch >= req.patch
            }
        }
    }
}

impl Manifest {
    /// Parse a slime.toml file from disk
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
        Self::parse(&content)
    }

    /// Parse slime.toml content (minimal TOML parser — zero deps)
    pub fn parse(content: &str) -> Result<Self, String> {
        let mut section = "";
        let mut name = String::new();
        let mut version = String::from("0.1.0");
        let mut authors: Vec<String> = Vec::new();
        let mut description: Option<String> = None;
        let mut license: Option<String> = None;
        let mut entry = String::from("src/main.slime");
        let mut dependencies: HashMap<String, VersionReq> = HashMap::new();
        let mut dev_dependencies: HashMap<String, VersionReq> = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') {
                section = match line {
                    "[package]" => "package",
                    "[dependencies]" => "dependencies",
                    "[dev-dependencies]" => "dev_dependencies",
                    _ => "",
                };
                continue;
            }

            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim();
                let val = val.trim().trim_matches('"');

                match section {
                    "package" => match key {
                        "name"        => name = val.to_string(),
                        "version"     => version = val.to_string(),
                        "description" => description = Some(val.to_string()),
                        "license"     => license = Some(val.to_string()),
                        "entry"       => entry = val.to_string(),
                        "authors"     => {
                            // parse ["author1", "author2"]
                            authors = val
                                .trim_matches(|c| c == '[' || c == ']')
                                .split(',')
                                .map(|a| a.trim().trim_matches('"').to_string())
                                .collect();
                        }
                        _ => {}
                    },
                    "dependencies" => {
                        let req = VersionReq::parse(val)?;
                        dependencies.insert(key.to_string(), req);
                    }
                    "dev_dependencies" => {
                        let req = VersionReq::parse(val)?;
                        dev_dependencies.insert(key.to_string(), req);
                    }
                    _ => {}
                }
            }
        }

        if name.is_empty() {
            return Err("slime.toml: missing [package] name".to_string());
        }

        Ok(Manifest {
            package: PackageMeta {
                name,
                version: Version::parse(&version)?,
                authors,
                description,
                license,
                entry,
            },
            dependencies,
            dev_dependencies,
        })
    }

    /// Serialize back to slime.toml format
    pub fn to_toml(&self) -> String {
        let mut out = String::new();
        out.push_str("[package]\n");
        out.push_str(&format!("name = \"{}\"\n", self.package.name));
        out.push_str(&format!("version = \"{}\"\n", self.package.version.to_string()));
        if !self.package.authors.is_empty() {
            let authors = self.package.authors
                .iter()
                .map(|a| format!("\"{}\"", a))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("authors = [{}]\n", authors));
        }
        if let Some(ref desc) = self.package.description {
            out.push_str(&format!("description = \"{}\"\n", desc));
        }
        if let Some(ref lic) = self.package.license {
            out.push_str(&format!("license = \"{}\"\n", lic));
        }
        out.push_str(&format!("entry = \"{}\"\n", self.package.entry));

        if !self.dependencies.is_empty() {
            out.push_str("\n[dependencies]\n");
            for (name, req) in &self.dependencies {
                out.push_str(&format!("{} = \"{}\"\n", name, req.display()));
            }
        }

        if !self.dev_dependencies.is_empty() {
            out.push_str("\n[dev-dependencies]\n");
            for (name, req) in &self.dev_dependencies {
                out.push_str(&format!("{} = \"{}\"\n", name, req.display()));
            }
        }

        out
    }
}

impl VersionReq {
    pub fn display(&self) -> String {
        match self {
            VersionReq::Any => "*".to_string(),
            VersionReq::Exact(v) => format!("={}", v.to_string()),
            VersionReq::Caret(v) => format!("^{}", v.to_string()),
            VersionReq::Tilde(v) => format!("~{}", v.to_string()),
        }
    }
}
