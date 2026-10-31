use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

const DEPENDENCIES_START: &str = "<!-- dependencies:start -->";
const DEPENDENCIES_END: &str = "<!-- dependencies:end -->";

#[derive(Debug, Deserialize)]
struct Manifest {
    schema: u32,
    asset: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    kind: String,
    path: String,
    sha256: String,
    license: String,
    license_path: String,
    license_sha256: String,
    source: String,
    source_version: String,
    evidence: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("assets") => {
            let check = args.iter().any(|arg| arg == "--check");
            if args.len() > 1 && !check {
                return usage();
            }
            run_assets(check)
        }
        Some("notices") => {
            let check = args.iter().any(|arg| arg == "--check");
            if args.len() > 1 && !check {
                return usage();
            }
            run_notices(check)
        }
        _ => usage(),
    }
}

fn usage() -> Result<(), Box<dyn Error>> {
    eprintln!("uso: cargo xtask assets [--check]");
    eprintln!("     cargo xtask notices [--check]");
    std::process::exit(2);
}

fn run_assets(check: bool) -> Result<(), Box<dyn Error>> {
    let root = workspace_root();
    let manifest_path = root.join("assets/manifest.toml");
    let text = std::fs::read_to_string(&manifest_path)
        .map_err(|error| format!("{}: {error}", manifest_path.display()))?;
    let manifest: Manifest = toml::from_str(&text)?;

    if manifest.schema != 1 {
        return Err(format!("schema de manifesto não suportado: {}", manifest.schema).into());
    }
    if manifest.asset.is_empty() {
        return Err("manifesto sem assets".into());
    }

    for asset in &manifest.asset {
        verify_asset(&root, asset)?;
        if !check {
            println!(
                "assets: {} ({}) verificado — {} {}",
                asset.name, asset.kind, asset.source_version, asset.license
            );
        }
    }

    if check {
        println!(
            "xtask assets --check: {} asset(s) em dia",
            manifest.asset.len()
        );
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct Lockfile {
    package: Vec<LockPackage>,
}

#[derive(Debug, Deserialize)]
struct LockPackage {
    name: String,
    version: String,
    source: Option<String>,
}

fn run_notices(check: bool) -> Result<(), Box<dyn Error>> {
    let root = workspace_root();
    let lock_path = root.join("Cargo.lock");
    let text = std::fs::read_to_string(&lock_path)
        .map_err(|error| format!("{}: {error}", lock_path.display()))?;
    let lock: Lockfile = toml::from_str(&text)?;

    let mut expected: Vec<(String, String)> = lock
        .package
        .into_iter()
        .filter(|package| package.source.is_some())
        .map(|package| (package.name, package.version))
        .collect();
    expected.sort();
    expected.dedup();

    let notices_path = root.join("THIRD_PARTY_NOTICES.md");
    let notices = std::fs::read_to_string(&notices_path)
        .map_err(|error| format!("{}: {error}", notices_path.display()))?;
    let actual = parse_notice_rows(&notices)?;
    let missed: Vec<_> = expected
        .iter()
        .filter(|entry| !actual.contains(entry))
        .collect();
    let stale: Vec<_> = actual
        .iter()
        .filter(|entry| !expected.contains(entry))
        .collect();

    if !missed.is_empty() || !stale.is_empty() {
        for (name, version) in &missed {
            eprintln!("ausente em THIRD_PARTY_NOTICES.md: {name} {version}");
        }
        for (name, version) in &stale {
            eprintln!("sobrando em THIRD_PARTY_NOTICES.md: {name} {version}");
        }
        return Err("THIRD_PARTY_NOTICES.md divergente do Cargo.lock".into());
    }

    if !check {
        println!(
            "notices: {} dependência(s) de Cargo.lock conferidas",
            expected.len()
        );
    } else {
        println!(
            "xtask notices --check: {} dependência(s) em dia",
            expected.len()
        );
    }
    Ok(())
}

fn parse_notice_rows(notices: &str) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let mut inside = false;
    let mut rows = Vec::new();
    for line in notices.lines() {
        let trimmed = line.trim();
        if trimmed == DEPENDENCIES_START {
            inside = true;
            continue;
        }
        if trimmed == DEPENDENCIES_END {
            inside = false;
            continue;
        }
        if !inside {
            continue;
        }
        let cells: Vec<&str> = trimmed
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        if cells.len() < 3 {
            continue;
        }
        let (Some(name), Some(version)) = (
            cells[0]
                .strip_prefix('`')
                .and_then(|cell| cell.strip_suffix('`')),
            cells[1]
                .strip_prefix('`')
                .and_then(|cell| cell.strip_suffix('`')),
        ) else {
            continue;
        };
        rows.push((name.to_string(), version.to_string()));
    }
    if rows.is_empty() {
        return Err("THIRD_PARTY_NOTICES.md sem bloco de dependências".into());
    }
    Ok(rows)
}

fn verify_asset(root: &Path, asset: &Asset) -> Result<(), Box<dyn Error>> {
    for (field, value) in [
        ("source", &asset.source),
        ("license", &asset.license),
        ("evidence", &asset.evidence),
    ] {
        if value.trim().is_empty() {
            return Err(format!("{}: campo {field} vazio", asset.name).into());
        }
    }

    let path = root.join(&asset.path);
    let actual = sha256_file(&path)?;
    if actual != asset.sha256 {
        return Err(format!(
            "{}: hash divergente em {} (esperado {}, encontrado {})",
            asset.name, asset.path, asset.sha256, actual
        )
        .into());
    }

    let license = root.join(&asset.license_path);
    let actual = sha256_file(&license)?;
    if actual != asset.license_sha256 {
        return Err(format!(
            "{}: hash divergente em {} (esperado {}, encontrado {})",
            asset.name, asset.license_path, asset.license_sha256, actual
        )
        .into());
    }

    if !path.is_file() || !license.is_file() {
        return Err(format!("{}: asset ou licença ausente", asset.name).into());
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask must live inside the workspace")
        .to_path_buf()
}
