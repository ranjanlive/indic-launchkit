pub mod apk;
pub mod render;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

pub type Result<T> = std::result::Result<T, String>;
pub const FILE_NAME: &str = "launchkit.json";
pub const MAX_FILE: u64 = 64 * 1024 * 1024;
pub const MAX_APK: u64 = 1024 * 1024 * 1024;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Screenshot {
    pub file: String,
    #[serde(default)]
    pub caption: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub schema_version: u32,
    pub title: String,
    #[serde(default)]
    pub tagline: String,
    #[serde(default)]
    pub package_name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_template")]
    pub template: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub apk: Option<String>,
    #[serde(default)]
    pub screenshots: Vec<Screenshot>,
}
fn default_version() -> String {
    "0.1.0".into()
}
fn default_template() -> String {
    "aurora".into()
}

#[derive(Debug, Clone, Serialize)]
pub struct Artifact {
    pub file: String,
    pub bytes: u64,
    pub sha256: String,
}
#[derive(Debug, Serialize)]
pub struct ExportReport {
    pub schema_version: u32,
    pub output: String,
    pub dry_run: bool,
    pub artifacts: Vec<Artifact>,
}

pub fn hash_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
pub fn read_limited(path: &Path, max: u64) -> Result<Vec<u8>> {
    if fs::metadata(path).map_err(|e| e.to_string())?.len() > max {
        return Err(format!("File exceeds limit: {}", path.display()));
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > max {
        return Err("Input grew past its size limit".into());
    }
    Ok(bytes)
}
pub fn asset(root: &Path, relative: &str) -> Result<PathBuf> {
    let path = Path::new(relative);
    if relative.is_empty()
        || relative.contains('\\')
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("Asset paths must be relative, without traversal".into());
    }
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut current = root.clone();
    for part in path.components() {
        current.push(part.as_os_str());
        if fs::symlink_metadata(&current)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
        {
            return Err("Symlink assets are not supported".into());
        }
    }
    let canonical = current.canonicalize().map_err(|e| e.to_string())?;
    if !canonical.starts_with(&root) || !canonical.is_file() {
        return Err("Asset escapes the project or is not a file".into());
    }
    Ok(canonical)
}
pub fn load(root: &Path) -> Result<Project> {
    let bytes = read_limited(&root.join(FILE_NAME), 1024 * 1024)?;
    let project: Project =
        serde_json::from_slice(&bytes).map_err(|e| format!("Invalid project JSON: {e}"))?;
    validate(root, &project, false)?;
    Ok(project)
}
pub fn save(root: &Path, project: &Project) -> Result<()> {
    validate(root, project, false)?;
    let mut file = tempfile::NamedTempFile::new_in(root).map_err(|e| e.to_string())?;
    file.write_all(&serde_json::to_vec_pretty(project).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    file.as_file().sync_all().map_err(|e| e.to_string())?;
    file.persist(root.join(FILE_NAME))
        .map_err(|e| e.to_string())?;
    Ok(())
}
pub fn init(root: &Path, title: &str, package: &str, template: &str) -> Result<Project> {
    if root.exists() {
        return Err("Project destination already exists; choose a new directory".into());
    }
    let project = Project {
        schema_version: 1,
        title: title.into(),
        tagline: String::new(),
        package_name: package.into(),
        version: default_version(),
        template: template.into(),
        icon: None,
        apk: None,
        screenshots: vec![],
    };
    validate_fields(&project)?;
    fs::create_dir(root).map_err(|e| e.to_string())?;
    fs::create_dir(root.join("assets")).map_err(|e| e.to_string())?;
    save(root, &project)?;
    Ok(project)
}
pub fn import(root: &Path, source: &Path, kind: &str, caption: &str) -> Result<Project> {
    let mut project = load(root)?;
    if caption.chars().count() > 180 || caption.chars().any(|c| c.is_control() && c != '\n') {
        return Err(
            "Caption must contain at most 180 characters without control characters".into(),
        );
    }
    render::validate_text(caption)?;
    if source.is_symlink() || !source.is_file() {
        return Err("Choose a regular file".into());
    }
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if kind == "apk" {
        if ext != "apk" {
            return Err("Choose an APK".into());
        }
    } else {
        if !["png", "jpg", "jpeg", "webp"].contains(&ext.as_str()) {
            return Err("Choose PNG, JPEG or WebP".into());
        }
        render::decode(source)?;
    }
    if kind == "screenshot" && project.screenshots.len() >= 20 {
        return Err("At most 20 screenshots are supported".into());
    }
    if !["icon", "apk", "screenshot"].contains(&kind) {
        return Err("Unknown asset kind".into());
    }
    if fs::metadata(source).map_err(|e| e.to_string())?.len()
        > if kind == "apk" { MAX_APK } else { MAX_FILE }
    {
        return Err("Input file exceeds its size limit".into());
    }
    let inspection = if kind == "apk" {
        Some(apk::inspect(source)?)
    } else {
        None
    };
    let hash = hash_file(source)?;
    let relative = format!("assets/{kind}-{}.{}", &hash[..20], ext);
    let destination = root.join(&relative);
    if root.join("assets").is_symlink() {
        return Err("Asset directory cannot be a symlink".into());
    }
    if destination.is_symlink() {
        return Err("Asset destination cannot be a symlink".into());
    }
    if !destination.exists() {
        let limit = if kind == "apk" { MAX_APK } else { MAX_FILE };
        let mut staged =
            tempfile::NamedTempFile::new_in(root.join("assets")).map_err(|e| e.to_string())?;
        let copied = std::io::copy(
            &mut fs::File::open(source)
                .map_err(|e| e.to_string())?
                .take(limit + 1),
            &mut staged,
        )
        .map_err(|e| e.to_string())?;
        if copied > limit || hash_file(staged.path())? != hash {
            return Err("Source changed or exceeded its size limit during import".into());
        }
        staged.as_file().sync_all().map_err(|e| e.to_string())?;
        staged
            .persist_noclobber(&destination)
            .map_err(|e| e.to_string())?;
    }
    if hash_file(&destination)? != hash {
        return Err("Imported file hash mismatch".into());
    }
    match kind {
        "icon" => project.icon = Some(relative),
        "apk" => {
            project.apk = Some(relative);
            if let Some(info) = inspection {
                project.package_name = info.package_name;
                if let Some(v) = info.version_name {
                    project.version = v;
                }
            }
        }
        _ => project.screenshots.push(Screenshot {
            file: relative,
            caption: caption.into(),
        }),
    }
    save(root, &project)?;
    Ok(project)
}
fn validate_fields(p: &Project) -> Result<()> {
    if p.schema_version != 1 {
        return Err("Unsupported project schema_version".into());
    }
    if p.title.trim().is_empty() || p.title.chars().count() > 64 {
        return Err("Title must contain 1–64 characters".into());
    }
    if p.tagline.chars().count() > 140 || p.screenshots.len() > 20 {
        return Err("Tagline or screenshot limit exceeded".into());
    }
    if !["aurora", "paper", "midnight"].contains(&p.template.as_str()) {
        return Err("Unknown template".into());
    }
    if p.version.is_empty() || p.version.chars().count() > 64 {
        return Err("Version must contain 1–64 characters".into());
    }
    if p.package_name.len() > 255
        || !p
            .package_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_')
    {
        return Err("Package name must use ASCII letters, digits, dots and underscores".into());
    }
    for text in std::iter::once(&p.title)
        .chain(std::iter::once(&p.tagline))
        .chain(p.screenshots.iter().map(|s| &s.caption))
    {
        if text.chars().any(|c| c.is_control() && c != '\n') {
            return Err("Control characters are not allowed".into());
        }
        if text.chars().count() > 180 {
            return Err("Screenshot captions must be at most 180 characters".into());
        }
        render::validate_text(text)?;
    }
    Ok(())
}
pub fn validate(root: &Path, p: &Project, require_assets: bool) -> Result<()> {
    validate_fields(p)?;
    if require_assets && (p.icon.is_none() || p.screenshots.is_empty()) {
        return Err("Add an icon and at least one screenshot before exporting".into());
    }
    if let Some(icon) = &p.icon {
        render::decode(&asset(root, icon)?)?;
    }
    for screenshot in &p.screenshots {
        render::decode(&asset(root, &screenshot.file)?)?;
    }
    if let Some(apk) = &p.apk {
        let path = asset(root, apk)?;
        if fs::metadata(path).map_err(|e| e.to_string())?.len() > MAX_APK {
            return Err("APK exceeds 1 GiB".into());
        }
    }
    Ok(())
}
pub fn export(
    root: &Path,
    p: &Project,
    output: &Path,
    dry_run: bool,
    include_apk: bool,
) -> Result<ExportReport> {
    validate(root, p, true)?;
    if output.exists() || output.is_symlink() {
        return Err(
            "Export destination already exists; existing files are never overwritten".into(),
        );
    }
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    if !parent.is_dir() {
        return Err("Output parent must already exist".into());
    }
    let names = std::iter::once("assets/icon.png".to_string())
        .chain([
            "assets/feature-graphic.png".into(),
            "assets/social-card.png".into(),
        ])
        .chain((0..p.screenshots.len()).map(|i| format!("assets/screenshots/{:02}.png", i + 1)))
        .collect::<Vec<_>>();
    if include_apk && p.apk.is_none() {
        return Err("No APK is selected".into());
    }
    // Preflight layout and APK parsing in both modes, without writing any files.
    render::check_layout(p)?;
    let apk = if let Some(file) = &p.apk {
        Some(apk::inspect(&asset(root, file)?)?)
    } else {
        None
    };
    if dry_run {
        let mut planned = names.clone();
        if include_apk {
            planned.push("release/builds/android/app.apk".into())
        }
        planned.extend([
            "release/metadata.json".into(),
            "docs/RELEASE_CHECKLIST.md".into(),
            "SHA256SUMS.txt".into(),
        ]);
        return Ok(ExportReport {
            schema_version: 1,
            output: output.display().to_string(),
            dry_run: true,
            artifacts: planned
                .into_iter()
                .map(|file| Artifact {
                    file,
                    bytes: 0,
                    sha256: String::new(),
                })
                .collect(),
        });
    }
    let destination_name = output
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Output must name a new directory")?;
    let lock_path = parent.join(format!(".{destination_name}.launchkit-lock"));
    let _lock = ExportLock {
        file: fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
            .map_err(|e| format!("Could not reserve export destination: {e}"))?,
        path: lock_path,
    };
    let stage = tempfile::Builder::new()
        .prefix(".launchkit-export-")
        .tempdir_in(parent)
        .map_err(|e| e.to_string())?;
    fs::create_dir_all(stage.path().join("assets/screenshots")).map_err(|e| e.to_string())?;
    let mut artifacts = vec![];
    for file in &names {
        let kind = if file.ends_with("icon.png") {
            "icon"
        } else if file.ends_with("feature-graphic.png") {
            "feature"
        } else if file.ends_with("social-card.png") {
            "social"
        } else {
            "screenshot"
        };
        let index = if kind == "screenshot" {
            file.rsplit('/')
                .next()
                .unwrap()
                .trim_end_matches(".png")
                .parse::<usize>()
                .unwrap()
                - 1
        } else {
            0
        };
        let bytes = render::png(root, p, kind, index, false)?;
        fs::write(stage.path().join(file), bytes).map_err(|e| e.to_string())?;
        artifacts.push(record(stage.path(), file)?);
    }
    if include_apk {
        let file = p.apk.as_ref().ok_or("No APK is selected")?;
        fs::create_dir_all(stage.path().join("release/builds/android"))
            .map_err(|e| e.to_string())?;
        fs::copy(
            asset(root, file)?,
            stage.path().join("release/builds/android/app.apk"),
        )
        .map_err(|e| e.to_string())?;
        artifacts.push(record(stage.path(), "release/builds/android/app.apk")?);
    }
    fs::create_dir_all(stage.path().join("docs")).map_err(|e| e.to_string())?;
    fs::create_dir_all(stage.path().join("release")).map_err(|e| e.to_string())?;
    let metadata = serde_json::json!({"schema_version":1,"generator":{"name":"Indic LaunchKit","version":VERSION},"product":{"title":p.title,"tagline":p.tagline,"package_name":p.package_name,"version":p.version},"template":p.template,"apk":apk,"artifacts":artifacts,"signing_verified":false,"store_publication_verified":false});
    fs::write(
        stage.path().join("release/metadata.json"),
        serde_json::to_vec_pretty(&metadata).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(stage.path().join("docs/RELEASE_CHECKLIST.md"),"# Review before publication\n\n- [ ] Review all captions and real screenshots.\n- [ ] Review store image requirements and licensing.\n- [ ] Verify the exact binary and signing identity.\n- [ ] Complete the product-specific privacy notice.\n- [ ] Verify download URLs after upload.\n\nGenerated assets do not certify store acceptance or legal compliance.\n").map_err(|e|e.to_string())?;
    artifacts.push(record(stage.path(), "release/metadata.json")?);
    artifacts.push(record(stage.path(), "docs/RELEASE_CHECKLIST.md")?);
    let sums = artifacts
        .iter()
        .map(|a| format!("{}  {}\n", a.sha256, a.file))
        .collect::<String>();
    fs::write(stage.path().join("SHA256SUMS.txt"), sums).map_err(|e| e.to_string())?;
    artifacts.push(record(stage.path(), "SHA256SUMS.txt")?);
    // Re-check immediately before atomic publication; never replace an existing path.
    if output.exists() || output.is_symlink() {
        return Err("Export destination appeared during rendering".into());
    }
    fs::rename(stage.path(), output).map_err(|e| e.to_string())?;
    Ok(ExportReport {
        schema_version: 1,
        output: output.display().to_string(),
        dry_run: false,
        artifacts,
    })
}
struct ExportLock {
    file: fs::File,
    path: PathBuf,
}
impl Drop for ExportLock {
    fn drop(&mut self) {
        let _ = self.file.sync_all();
        let _ = fs::remove_file(&self.path);
    }
}
fn record(root: &Path, file: &str) -> Result<Artifact> {
    let path = root.join(file);
    Ok(Artifact {
        file: file.into(),
        bytes: fs::metadata(&path).map_err(|e| e.to_string())?.len(),
        sha256: hash_file(&path)?,
    })
}

pub fn schema() -> serde_json::Value {
    serde_json::json!({"$schema":"https://json-schema.org/draft/2020-12/schema","title":"Indic LaunchKit project","type":"object","additionalProperties":false,"required":["schema_version","title"],"properties":{"schema_version":{"const":1},"title":{"type":"string","minLength":1,"maxLength":64},"tagline":{"type":"string","maxLength":140},"package_name":{"type":"string"},"version":{"type":"string","minLength":1,"maxLength":64},"template":{"enum":["aurora","paper","midnight"]},"icon":{"type":["string","null"]},"apk":{"type":["string","null"]},"screenshots":{"type":"array","maxItems":20,"items":{"type":"object","additionalProperties":false,"required":["file"],"properties":{"file":{"type":"string"},"caption":{"type":"string","maxLength":180}}}}}})
}
