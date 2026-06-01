use anyhow::{Result, bail, Context};
use std::fs::{self, File};
use std::io::{Write, Read};
use std::path::Path;
use serde::{Serialize, Deserialize};
use serde_json::json;
use zip::write::FileOptions;
use std::env;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ReleaseInfo {
    tag_name: String,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    published_at: String,
    id: i64,
    url: Option<String>,
    assets: Option<Vec<AssetInfo>>,
}

#[derive(Debug, Deserialize)]
struct AssetInfo {
    id: i64,
    name: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct TsNativeToml {
    package: TomlPackage,
}

#[derive(Debug, Deserialize)]
struct TomlPackage {
    version: Option<String>,
    #[allow(dead_code)]
    tsnp_version: Option<String>,
}

#[derive(Serialize)]
struct FileInfo {
    path: String,
    content: String,
}

fn get_codeberg_token() -> Result<String> {
    env::var("CODEBERG_TOKEN")
        .or_else(|_| env::var("GITEA_TOKEN"))
        .context("CODEBERG_TOKEN or GITEA_TOKEN environment variable not set")
}

fn get_codeberg_user() -> String {
    env::var("CODEBERG_USER").unwrap_or_else(|_| "tsnp".to_string())
}

fn get_codeberg_repo() -> String {
    env::var("CODEBERG_REPO").unwrap_or_else(|_| "tsnp".to_string())
}

fn get_codeberg_api() -> String {
    env::var("CODEBERG_API").unwrap_or_else(|_| "https://codeberg.org/api/v1".to_string())
}

fn get_codeberg_base_url() -> String {
    get_codeberg_api().replace("/api/v1", "")
}

pub fn cmd_list() {
    println!("Listing local plugins:");
    
    let tsnp_dir = Path::new("tsnp");
    if !tsnp_dir.exists() {
        println!("No tsnp/ directory found.");
        return;
    }
    
    for entry in fs::read_dir(tsnp_dir).unwrap().filter_map(|e| e.ok()) {
        if entry.path().is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                let toml_path = entry.path().join("ts-native.toml");
                if toml_path.exists() {
                    if let Ok(version) = extract_version_from_toml(&toml_path) {
                        println!("  - {} v{}", name, version);
                    } else {
                        println!("  - {} (error reading toml)", name);
                    }
                }
            }
        }
    }
}

fn extract_version_from_toml(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)?;
    let toml: TsNativeToml = toml::from_str(&content)?;
    Ok(toml.package.version.unwrap_or_else(|| "0.1.0".to_string()))
}

#[allow(dead_code)]
fn extract_version_from_str(content: &str) -> String {
    toml::from_str::<TsNativeToml>(content)
        .ok()
        .and_then(|t| t.package.version)
        .unwrap_or_else(|| "0.1.0".to_string())
}

pub fn cmd_publish(dry_run: bool) {
    if let Err(e) = cmd_publish_inner(dry_run) {
        eprintln!("Failed to publish: {:#}", e);
    }
}

fn cmd_publish_inner(dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Dry-run mode - no files will be uploaded");
    } else {
        println!("Publishing plugins to codeberg.org");
    }
    
    let token = get_codeberg_token()?;
    let user = get_codeberg_user();
    let repo = get_codeberg_repo();
    println!("Target: {}/{}/{}", get_codeberg_base_url(), user, repo);
    
    let tsnp_dir = Path::new("tsnp");
    if !tsnp_dir.exists() {
        bail!("No tsnp/ directory found.");
    }
    
    let mut plugins = Vec::new();
    for entry in fs::read_dir(tsnp_dir)?.filter_map(|e| e.ok()) {
        if entry.path().is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                let toml_path = entry.path().join("ts-native.toml");
                if toml_path.exists() {
                    plugins.push(name.to_string());
                }
            }
        }
    }
    
    if plugins.is_empty() {
        bail!("No plugins found in tsnp/ directory.");
    }
    
    println!("\nAvailable plugins:");
    for (i, plugin) in plugins.iter().enumerate() {
        println!("[{}] {}", i + 1, plugin);
    }
    println!("[a] Publish all");
    println!("[q] Cancel");
    
    print!("\nSelect: ");
    std::io::stdout().flush()?;
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim();
    
    let selected = match input {
        "q" => {
            println!("Cancelled.");
            return Ok(());
        }
        "a" => plugins,
        _ => {
            if let Ok(idx) = input.parse::<usize>() {
                if idx >= 1 && idx <= plugins.len() {
                    vec![plugins[idx-1].clone()]
                } else {
                    bail!("Invalid selection.");
                }
            } else {
                bail!("Invalid selection.");
            }
        }
    };
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    for plugin in &selected {
        println!("\n[{}/{}] Publishing {}...", success_count + fail_count + 1, selected.len(), plugin);
        
        match publish_plugin(plugin, &token, dry_run) {
            Ok(_) => {
                if dry_run {
                    println!("{} would be published successfully.", plugin);
                } else {
                    println!("Published {} successfully.", plugin);
                }
                success_count += 1;
            }
            Err(e) => {
                eprintln!("Failed to publish {}: {:#}", plugin, e);
                fail_count += 1;
            }
        }
    }
    
    println!("\nSummary: {} succeeded, {} failed", success_count, fail_count);
    
    Ok(())
}

fn publish_plugin(plugin_name: &str, token: &str, dry_run: bool) -> Result<()> {
    let plugin_dir = Path::new("tsnp").join(plugin_name);
    
    if !plugin_dir.exists() {
        bail!("Plugin directory {} does not exist", plugin_dir.display());
    }
    
    let ts_toml = plugin_dir.join("ts-native.toml");
    if !ts_toml.exists() {
        bail!("ts-native.toml not found in {}", plugin_dir.display());
    }
    
    let version = extract_version_from_toml(&ts_toml)?;
    let author = env::var("CODEBERG_AUTHOR").unwrap_or_else(|_| "tsnp".to_string());
    
    println!("   Version: {}, Author: {}", version, author);
    
    let mut files = Vec::new();
    collect_files(&plugin_dir, &mut files, plugin_name)?;
    
    if files.is_empty() {
        bail!("No files found to publish");
    }
    
    println!("   Files: {}", files.len());
    
    if dry_run {
        println!("   [DRY RUN] Would create zip file: {}-{}.zip", plugin_name, version);
        println!("   [DRY RUN] Would upload to: {}/{}/{}/releases/tag/{}-{}", 
            get_codeberg_base_url(), get_codeberg_user(), get_codeberg_repo(), plugin_name, version);
        return Ok(());
    }
    
    let zip_path = format!("{}-{}.zip", plugin_name, version);
    create_zip(&zip_path, &files)?;
    
    let mut zip_data = Vec::new();
    let mut file = File::open(&zip_path)?;
    file.read_to_end(&mut zip_data)?;
    
    upload_to_codeberg(plugin_name, &version, &zip_data, &author, token)?;
    
    if let Err(e) = fs::remove_file(&zip_path) {
        eprintln!("Warning: Failed to remove temporary file {}: {}", zip_path, e);
    }
    
    Ok(())
}

fn collect_files(dir: &Path, files: &mut Vec<FileInfo>, _base_path: &str) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name()
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Non-UTF8 filename in {}", dir.display()))?
            .to_string();
        
        if name == "target" || name == ".git" || name == ".gitignore" {
            continue;
        }
        
        let rel_path = path.strip_prefix("tsnp")
            .map_err(|_| anyhow::anyhow!("Path {} is not under tsnp/", path.display()))?;
        let rel_str = rel_path.to_str()
            .ok_or_else(|| anyhow::anyhow!("Non-UTF8 path: {}", rel_path.display()))?
            .replace("\\", "/");
        
        if path.is_dir() {
            collect_files(&path, files, _base_path)?;
        } else {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            files.push(FileInfo {
                path: rel_str,
                content,
            });
        }
    }
    Ok(())
}

fn create_zip(zip_path: &str, files: &[FileInfo]) -> Result<()> {
    let file = File::create(zip_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options: FileOptions<'_, ()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    
    for file_info in files {
        zip.start_file(&file_info.path, options)?;
        zip.write_all(file_info.content.as_bytes())?;
    }
    
    zip.finish()?;
    Ok(())
}

fn upload_to_codeberg(name: &str, version: &str, zip_data: &[u8], author: &str, token: &str) -> Result<()> {
    let api_base = get_codeberg_api();
    let user = get_codeberg_user();
    let repo = get_codeberg_repo();
    let tag_name = format!("{}-{}", name, version);
    
    let check_url = format!("{}/repos/{}/{}/releases/tags/{}", api_base, user, repo, tag_name);
    let existing_release = ureq::get(&check_url)
        .set("Authorization", &format!("token {}", token))
        .call();
    
    let (release_id, release_url) = match existing_release {
        Ok(resp) if resp.status() == 200 => {
            println!("   Release already exists, updating...");
            let body = resp.into_string()?;
            let release_info: serde_json::Value = serde_json::from_str(&body)?;
            let id = release_info["id"].as_i64().ok_or_else(|| anyhow::anyhow!("No release ID found"))?;
            let url = release_info["url"].as_str().unwrap_or("").to_string();
            (id, url)
        }
        _ => {
            let create_url = format!("{}/repos/{}/{}/releases", api_base, user, repo);
            let release_data = json!({
                "tag_name": tag_name,
                "target_commitish": "main",
                "name": format!("{} v{}", name, version),
                "body": format!("Plugin: {}\nVersion: {}\nAuthor: {}", name, version, author),
                "draft": false,
                "prerelease": false,
                "make_latest": "true"
            });
            
            println!("   Creating release...");
            let resp = ureq::post(&create_url)
                .set("Authorization", &format!("token {}", token))
                .set("Content-Type", "application/json")
                .send_string(&release_data.to_string())?;
            
            if resp.status() != 201 {
                let status = resp.status();
                let status_text = resp.status_text().to_string();
                let body = resp.into_string().unwrap_or_default();
                bail!("Failed to create release: {} {}\n{}", status, status_text, body);
            }
            
            let body = resp.into_string()?;
            let release_info: serde_json::Value = serde_json::from_str(&body)?;
            let id = release_info["id"].as_i64().ok_or_else(|| anyhow::anyhow!("No release ID returned"))?;
            let url = release_info["url"].as_str().unwrap_or("").to_string();
            (id, url)
        }
    };
    
    // Delete existing assets with same name
    let assets_url = format!("{}/repos/{}/{}/releases/{}/assets", api_base, user, repo, release_id);
    if let Ok(resp) = ureq::get(&assets_url)
        .set("Authorization", &format!("token {}", token))
        .call()
    {
        if resp.status() == 200 {
            if let Ok(body) = resp.into_string() {
                if let Ok(assets) = serde_json::from_str::<Vec<AssetInfo>>(&body) {
                    let asset_name = format!("{}-{}.zip", name, version);
                    for asset in assets {
                        if asset.name == asset_name {
                            let delete_url = format!("{}/repos/{}/{}/releases/{}/assets/{}", 
                                api_base, user, repo, release_id, asset.id);
                            let _ = ureq::delete(&delete_url)
                                .set("Authorization", &format!("token {}", token))
                                .call();
                            println!("   Deleted old asset: {}", asset_name);
                        }
                    }
                }
            }
        }
    }
    
    let asset_name = format!("{}-{}.zip", name, version);
    let upload_url = format!("{}/repos/{}/{}/releases/{}/assets", 
        api_base, user, repo, release_id);
    
    println!("   Uploading {}...", asset_name);
    let resp = ureq::post(&upload_url)
        .set("Authorization", &format!("token {}", token))
        .set("Content-Type", "application/zip")
        .query("name", &asset_name)
        .send_bytes(zip_data)?;
    
    if resp.status() != 201 {
        let status = resp.status();
        let status_text = resp.status_text().to_string();
        let body = resp.into_string().unwrap_or_default();
        bail!("Failed to upload asset: {} {}\n{}", status, status_text, body);
    }
    
    if !release_url.is_empty() {
        let display_url = release_url.replace("/api/v1/repos/", "/").replace("/releases/", "/releases/tag/");
        println!("   Published: {}", display_url);
    } else {
        println!("   Published: {}/{}/{}/releases/tag/{}", 
            get_codeberg_base_url(), user, repo, tag_name);
    }
    
    Ok(())
}

pub fn fetch_published_tsnps() -> Result<Vec<(String, String, String)>> {
    let token = get_codeberg_token()?;
    let user = get_codeberg_user();
    let repo = get_codeberg_repo();
    let api_base = get_codeberg_api();
    
    let url = format!("{}/repos/{}/{}/releases", api_base, user, repo);
    
    let resp = ureq::get(&url)
        .set("Authorization", &format!("token {}", token))
        .call()?;
    
    if resp.status() != 200 {
        bail!("Failed to fetch releases: {} {}", resp.status(), resp.status_text());
    }
    
    let body = resp.into_string()?;
    let releases: Vec<ReleaseInfo> = serde_json::from_str(&body)?;
    
    let mut tsnps = Vec::new();
    for release in releases {
        let tag = &release.tag_name;
        if let Some(dash_pos) = tag.rfind('-') {
            let plugin_name = &tag[..dash_pos];
            let version = &tag[dash_pos + 1..];
            let publish_time = format_timestamp(&release.published_at);
            tsnps.push((plugin_name.to_string(), version.to_string(), publish_time));
        }
    }
    
    Ok(tsnps)
}

fn format_timestamp(iso_time: &str) -> String {
    if let Some(t_pos) = iso_time.find('T') {
        iso_time[..t_pos].to_string()
    } else {
        iso_time.to_string()
    }
}

pub fn download_tsnp(name: &str, version: Option<&str>) -> Result<()> {
    let token = get_codeberg_token()?;
    let user = get_codeberg_user();
    let repo = get_codeberg_repo();
    let api_base = get_codeberg_api();
    
    let tsnp_dir = Path::new("tsnp");
    fs::create_dir_all(tsnp_dir)?;
    
    let releases_url = format!("{}/repos/{}/{}/releases", api_base, user, repo);
    let resp = ureq::get(&releases_url)
        .set("Authorization", &format!("token {}", token))
        .call()?;
    
    if resp.status() != 200 {
        bail!("Failed to fetch releases: {} {}", resp.status(), resp.status_text());
    }
    
    let body = resp.into_string()?;
    let releases: Vec<ReleaseInfo> = serde_json::from_str(&body)?;
    
    let mut matching: Vec<&ReleaseInfo> = releases.iter()
        .filter(|r| {
            if let Some(dash_pos) = r.tag_name.rfind('-') {
                let plugin_name = &r.tag_name[..dash_pos];
                let ver = &r.tag_name[dash_pos + 1..];
                if plugin_name == name {
                    if let Some(v) = version {
                        return ver == v;
                    }
                    return true;
                }
            }
            false
        })
        .collect();
    
    if matching.is_empty() {
        if let Some(v) = version {
            bail!("No release found for {} v{}", name, v);
        } else {
            bail!("No release found for {}", name);
        }
    }
    
    matching.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    let release = matching[0];
    let tag_version = release.tag_name.rfind('-')
        .map(|pos| &release.tag_name[pos + 1..])
        .unwrap_or("unknown");
    
    let asset_name = format!("{}-{}.zip", name, tag_version);
    
    let assets = release.assets.as_ref().ok_or_else(|| anyhow::anyhow!("No assets in release"))?;
    let asset = assets.iter().find(|a| a.name == asset_name)
        .ok_or_else(|| anyhow::anyhow!("Asset {} not found in release", asset_name))?;
    
    println!("Downloading {} v{}...", name, tag_version);
    let download_resp = ureq::get(&asset.url)
        .set("Authorization", &format!("token {}", token))
        .call()?;
    
    if download_resp.status() != 200 {
        bail!("Failed to download asset: {} {}", download_resp.status(), download_resp.status_text());
    }
    
    let zip_data = download_resp.into_string()?.into_bytes();
    
    let dest_dir = tsnp_dir.join(name);
    fs::create_dir_all(&dest_dir)?;
    
    let reader = std::io::Cursor::new(&zip_data);
    let mut archive = zip::ZipArchive::new(reader)?;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = match file.enclosed_name() {
            Some(p) => dest_dir.join(p),
            None => continue,
        };
        
        if file.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = File::create(&out_path)?;
            std::io::copy(&mut file, &mut out_file)?;
        }
    }
    
    println!("Installed {} v{} to tsnp/{}/", name, tag_version, name);
    Ok(())
}
