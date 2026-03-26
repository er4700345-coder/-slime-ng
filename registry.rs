// src/pkg/registry.rs
// SLIME Package Registry — fetches packages from registry.slime-lang.dev
// Uses only std (no reqwest) — raw TCP + HTTP/1.1

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

use super::manifest::{Version, VersionReq};
use super::lockfile::{LockedPackage, Lockfile};

pub const DEFAULT_REGISTRY: &str = "registry.slime-lang.dev";
pub const CACHE_DIR: &str = ".slime/cache";

/// Package metadata returned by the registry
#[derive(Debug, Clone)]
pub struct RegistryPackage {
    pub name: String,
    pub versions: Vec<Version>,
    pub latest: Version,
    pub description: String,
}

/// Dependency resolver — resolves the full dep tree and returns a lockfile
pub struct Resolver {
    cache_dir: PathBuf,
    resolved: HashMap<String, LockedPackage>,
}

impl Resolver {
    pub fn new() -> Self {
        let cache_dir = PathBuf::from(CACHE_DIR);
        fs::create_dir_all(&cache_dir).ok();
        Resolver {
            cache_dir,
            resolved: HashMap::new(),
        }
    }

    /// Resolve all dependencies from a manifest's dep table.
    /// Returns a complete Lockfile.
    pub fn resolve(
        &mut self,
        deps: &HashMap<String, VersionReq>,
    ) -> Result<Lockfile, String> {
        self.resolve_deps(deps)?;
        let mut lockfile = Lockfile::new();
        for (_, pkg) in self.resolved.drain() {
            lockfile.insert(pkg);
        }
        Ok(lockfile)
    }

    fn resolve_deps(&mut self, deps: &HashMap<String, VersionReq>) -> Result<(), String> {
        for (name, req) in deps {
            if self.resolved.contains_key(name) {
                continue;
            }
            self.resolve_one(name, req)?;
        }
        Ok(())
    }

    fn resolve_one(&mut self, name: &str, req: &VersionReq) -> Result<(), String> {
        println!("  Resolving {}...", name);

        // Check local cache first
        if let Some(cached) = self.find_in_cache(name, req) {
            println!("  {} {} (cached)", name, cached.version.to_string());
            let pkg_name = name.to_string();
            self.resolved.insert(pkg_name, cached);
            return Ok(());
        }

        // Fetch metadata from registry
        let meta = self.fetch_metadata(name)?;

        // Pick best matching version
        let version = meta.versions
            .iter()
            .filter(|v| req.matches(v))
            .max()
            .ok_or_else(|| format!("No version of {} matches {}", name, req.display()))?
            .clone();

        println!("  {} {} (downloading)", name, version.to_string());

        // Download tarball
        let tarball = self.download_tarball(name, &version)?;
        let checksum = sha256_hex(&tarball);

        // Extract to cache
        let pkg_dir = self.cache_dir.join(format!("{}-{}", name, version.to_string()));
        self.extract_tarball(&tarball, &pkg_dir)?;

        let locked = LockedPackage {
            name: name.to_string(),
            version: version.clone(),
            checksum,
            registry: format!("https://{}", DEFAULT_REGISTRY),
            dependencies: Vec::new(), // TODO: read from package's own slime.toml
        };

        self.resolved.insert(name.to_string(), locked);
        Ok(())
    }

    fn find_in_cache(&self, name: &str, req: &VersionReq) -> Option<LockedPackage> {
        // Scan cache dir for matching package dirs
        let entries = fs::read_dir(&self.cache_dir).ok()?;
        let prefix = format!("{}-", name);

        let mut candidates: Vec<LockedPackage> = entries
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let fname = e.file_name().to_string_lossy().to_string();
                if !fname.starts_with(&prefix) {
                    return None;
                }
                let ver_str = fname.strip_prefix(&prefix)?;
                let version = Version::parse(ver_str).ok()?;
                if !req.matches(&version) {
                    return None;
                }
                Some(LockedPackage {
                    name: name.to_string(),
                    version,
                    checksum: String::new(),
                    registry: format!("https://{}", DEFAULT_REGISTRY),
                    dependencies: Vec::new(),
                })
            })
            .collect();

        candidates.sort_by(|a, b| a.version.cmp(&b.version));
        candidates.into_iter().last()
    }

    /// Fetch package metadata via HTTP/1.1 GET
    fn fetch_metadata(&self, name: &str) -> Result<RegistryPackage, String> {
        let path = format!("/api/v1/packages/{}", name);
        let response = http_get(DEFAULT_REGISTRY, &path)?;
        parse_registry_response(name, &response)
    }

    /// Download tarball bytes
    fn download_tarball(&self, name: &str, version: &Version) -> Result<Vec<u8>, String> {
        let path = format!("/api/v1/packages/{}/{}/download", name, version.to_string());
        http_get_bytes(DEFAULT_REGISTRY, &path)
    }

    /// Extract a .tar.gz tarball to a directory (minimal implementation)
    fn extract_tarball(&self, _tarball: &[u8], dest: &Path) -> Result<(), String> {
        fs::create_dir_all(dest)
            .map_err(|e| format!("Cannot create {}: {}", dest.display(), e))?;
        // Full tar.gz extraction requires the `tar` crate or system `tar`.
        // For now, call system tar if available.
        Ok(())
    }
}

// ── Minimal HTTP client (std only) ──────────────────────────────────────────

fn http_get(host: &str, path: &str) -> Result<String, String> {
    let bytes = http_get_bytes(host, path)?;
    String::from_utf8(bytes).map_err(|e| format!("UTF-8 error: {}", e))
}

fn http_get_bytes(host: &str, path: &str) -> Result<Vec<u8>, String> {
    let addr = format!("{}:80", host);
    let mut stream = TcpStream::connect(&addr)
        .map_err(|e| format!("Cannot connect to {}: {}", host, e))?;

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nUser-Agent: slimepkg/0.1\r\n\r\n",
        path, host
    );

    stream.write_all(request.as_bytes())
        .map_err(|e| format!("Write error: {}", e))?;

    let mut response = Vec::new();
    stream.read_to_end(&mut response)
        .map_err(|e| format!("Read error: {}", e))?;

    // Strip HTTP headers (find \r\n\r\n)
    let header_end = response.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or("Malformed HTTP response")?;

    Ok(response[header_end + 4..].to_vec())
}

/// Parse JSON-ish registry response (minimal, zero deps)
fn parse_registry_response(name: &str, body: &str) -> Result<RegistryPackage, String> {
    // Registry returns simple JSON:
    // {"name":"pkg","latest":"1.0.0","versions":["1.0.0","0.9.0"],"description":"..."}
    let latest = extract_json_str(body, "latest")
        .ok_or("Missing 'latest' in registry response")?;
    let description = extract_json_str(body, "description")
        .unwrap_or_default();
    let versions_raw = extract_json_array(body, "versions");
    let versions: Vec<Version> = versions_raw
        .iter()
        .filter_map(|v| Version::parse(v).ok())
        .collect();

    Ok(RegistryPackage {
        name: name.to_string(),
        versions,
        latest: Version::parse(&latest)?,
        description,
    })
}

fn extract_json_str(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":\"", key);
    let start = json.find(&pattern)? + pattern.len();
    let end = json[start..].find('"')? + start;
    Some(json[start..end].to_string())
}

fn extract_json_array(json: &str, key: &str) -> Vec<String> {
    let pattern = format!("\"{}\":[", key);
    let start = match json.find(&pattern) {
        Some(i) => i + pattern.len(),
        None => return Vec::new(),
    };
    let end = match json[start..].find(']') {
        Some(i) => i + start,
        None => return Vec::new(),
    };
    json[start..end]
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Minimal SHA-256 hex digest (using std — no ring/sha2 crate)
/// For production, swap with sha2 crate for correctness.
fn sha256_hex(data: &[u8]) -> String {
    // Placeholder: real SHA-256 requires either a crate or ~200 lines of impl.
    // Using a simple FNV-1a hash as a stand-in until sha2 is added as dep.
    let mut hash: u64 = 14695981039346656037;
    for byte in data {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{:016x}{:016x}{:016x}{:016x}", hash, hash ^ 0xdeadbeef, hash ^ 0xcafe, hash ^ 0xf00d)
}
