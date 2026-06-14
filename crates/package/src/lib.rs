use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Package not found: {0}")]
    NotFound(String),
    #[error("Version resolution failed for {0}")]
    VersionResolution(String),
    #[error("Invalid package name: {0}")]
    InvalidName(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackageJson {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub type_: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub dependencies: HashMap<String, String>,
}

impl Default for PackageJson {
    fn default() -> Self {
        Self {
            name: "project".to_string(),
            version: "1.0.0".to_string(),
            main: Some("main.js".to_string()),
            type_: Some("module".to_string()),
            dependencies: HashMap::new(),
        }
    }
}

pub struct PackageManager;

impl PackageManager {
    pub fn new() -> Self {
        Self
    }

    pub fn init_project(&self, name: &str, dir: &Path) -> Result<(), PackageError> {
        std::fs::create_dir_all(dir)?;

        let package_json = PackageJson {
            name: name.to_string(),
            main: Some("main.js".to_string()),
            type_: Some("module".to_string()),
            ..Default::default()
        };

        let package_json_path = dir.join("package.json");
        let content = serde_json::to_string_pretty(&package_json)?;
        std::fs::write(&package_json_path, content)?;

        let main_path = dir.join("main.js");
        if !main_path.exists() {
            std::fs::write(
                &main_path,
                r#"console.log("Hello from vole runtime!");
"#,
            )?;
        }

        println!("Initialized project '{}' in {}", name, dir.display());
        Ok(())
    }

    pub async fn install_packages(
        &self,
        packages: &[&str],
        dir: &Path,
    ) -> Result<(), PackageError> {
        let client = reqwest::Client::new();
        let node_modules = dir.join("node_modules");
        std::fs::create_dir_all(&node_modules)?;

        let package_json_path = dir.join("package.json");
        let mut package_json: PackageJson = if package_json_path.exists() {
            let content = std::fs::read_to_string(&package_json_path)?;
            serde_json::from_str(&content)?
        } else {
            PackageJson::default()
        };

        for package_spec in packages {
            let (name, version_req) = parse_package_spec(package_spec)?;
            println!("Installing {}@{}...", name, version_req);

            let version = resolve_version(&client, name, version_req).await?;
            println!("  Resolved version: {}", version);

            let tarball_url = fetch_tarball_url(&client, name, &version).await?;
            let bytes = client.get(&tarball_url).send().await?.bytes().await?;

            let package_dir = node_modules.join(name);
            if package_dir.exists() {
                std::fs::remove_dir_all(&package_dir)?;
            }
            std::fs::create_dir_all(&package_dir)?;

            extract_tarball(&bytes, &package_dir)?;

            package_json
                .dependencies
                .insert(name.to_string(), format!("^{}", version));
            println!("  Installed {}@{}", name, version);
        }

        let content = serde_json::to_string_pretty(&package_json)?;
        std::fs::write(&package_json_path, content)?;

        println!("Done.");
        Ok(())
    }

    pub async fn install_all(&self, dir: &Path) -> Result<(), PackageError> {
        let package_json_path = dir.join("package.json");
        if !package_json_path.exists() {
            return Err(PackageError::NotFound("package.json not found".to_string()));
        }

        let content = std::fs::read_to_string(&package_json_path)?;
        let package_json: PackageJson = serde_json::from_str(&content)?;

        if package_json.dependencies.is_empty() {
            println!("No dependencies to install.");
            return Ok(());
        }

        let specs: Vec<String> = package_json
            .dependencies
            .keys()
            .map(|k| k.clone())
            .collect();
        let refs: Vec<&str> = specs.iter().map(|s| s.as_str()).collect();

        self.install_packages(&refs, dir).await
    }
}

fn parse_package_spec(spec: &str) -> Result<(&str, &str), PackageError> {
    if let Some((name, version)) = spec.split_once('@') {
        if name.is_empty() {
            return Err(PackageError::InvalidName(spec.to_string()));
        }
        Ok((name, version))
    } else {
        Ok((spec, "latest"))
    }
}

async fn resolve_version(
    client: &reqwest::Client,
    name: &str,
    version_req: &str,
) -> Result<String, PackageError> {
    if version_req == "latest" {
        let url = format!("https://registry.npmjs.org/{}/latest", name);
        let resp: serde_json::Value = client.get(&url).send().await?.json().await?;
        let version = resp["version"]
            .as_str()
            .ok_or_else(|| PackageError::NotFound(name.to_string()))?;
        return Ok(version.to_string());
    }

    let url = format!("https://registry.npmjs.org/{}", name);
    let resp: serde_json::Value = client.get(&url).send().await?.json().await?;
    let versions = resp["versions"]
        .as_object()
        .ok_or_else(|| PackageError::NotFound(name.to_string()))?;

    let req = semver::VersionReq::parse(version_req)
        .map_err(|_| PackageError::VersionResolution(name.to_string()))?;

    let mut best: Option<semver::Version> = None;
    for v in versions.keys() {
        if let Ok(ver) = semver::Version::parse(v) {
            if req.matches(&ver) {
                if best.as_ref().map_or(true, |b| ver > *b) {
                    best = Some(ver);
                }
            }
        }
    }

    best.map(|v| v.to_string())
        .ok_or_else(|| PackageError::VersionResolution(name.to_string()))
}

async fn fetch_tarball_url(
    client: &reqwest::Client,
    name: &str,
    version: &str,
) -> Result<String, PackageError> {
    let url = format!("https://registry.npmjs.org/{}/{}", name, version);
    let resp: serde_json::Value = client.get(&url).send().await?.json().await?;
    let tarball = resp["dist"]["tarball"]
        .as_str()
        .ok_or_else(|| PackageError::NotFound(format!("{}@{}", name, version)))?;
    Ok(tarball.to_string())
}

fn extract_tarball(data: &[u8], dest: &Path) -> Result<(), PackageError> {
    let decoder = flate2::read::GzDecoder::new(data);
    let mut archive = tar::Archive::new(decoder);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        // Strip "package/" prefix
        let stripped = path.strip_prefix("package").unwrap_or(&path);
        let target = dest.join(stripped);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        entry.unpack(&target)?;
    }

    Ok(())
}
