use crate::catalog::{Offer, Product};
use crate::diagnostics::{Error, Result};
use crate::github::GithubClient;
use crate::versions::{Asset, Version};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn install(
    client: &GithubClient,
    version: &Version,
    offer: &Offer,
    destination: &Path,
) -> Result<()> {
    match offer.product {
        Product::MotorPyPip => install_python(client, offer, destination),
        Product::MotorPyProject => install_project(client, version, destination),
        Product::MotorRuCore => install_rust_core(client, version, destination),
        Product::MotorRuExecutable => install_executable(client, offer, destination),
    }
}

fn temporary_file(_asset: &Asset) -> Result<(tempfile::NamedTempFile, PathBuf)> {
    let temp =
        tempfile::NamedTempFile::new().map_err(|e| Error::new("INSTALL001", e.to_string()))?;
    let path = temp.path().to_path_buf();
    Ok((temp, path))
}

fn download_verified(client: &GithubClient, asset: &Asset) -> Result<PathBuf> {
    let (temp, path) = temporary_file(asset)?;
    let mut last_error = None;
    for _ in 0..3 {
        match client.download(&asset.url, &path) {
            Ok(size) => {
                if size != asset.size {
                    return Err(Error::new(
                        "VERIFY001",
                        format!("tamaño inesperado para {}", asset.name),
                    ));
                }
                if let Some(digest) = &asset.digest {
                    if let Some(expected) = digest.strip_prefix("sha256:") {
                        let actual = sha256(&path)?;
                        if !actual.eq_ignore_ascii_case(expected) {
                            return Err(Error::new(
                                "VERIFY002",
                                format!("SHA-256 no coincide para {}", asset.name),
                            ));
                        }
                    }
                }
                std::mem::forget(temp);
                return Ok(path);
            }
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| Error::new("DOWNLOAD001", "descarga fallida")))
}

fn sha256(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path).map_err(|e| Error::new("VERIFY001", e.to_string()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|e| Error::new("VERIFY001", e.to_string()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn install_python(client: &GithubClient, offer: &Offer, destination: &Path) -> Result<()> {
    let temp = tempfile::tempdir().map_err(|e| Error::new("INSTALL001", e.to_string()))?;
    let mut packages = Vec::new();
    for asset in &offer.assets {
        if asset.name.starts_with("pixl_motorpy") {
            let path = download_verified(client, asset)?;
            let target = temp.path().join(&asset.name);
            std::fs::copy(&path, &target).map_err(|e| Error::new("INSTALL002", e.to_string()))?;
            let _ = std::fs::remove_file(path);
            packages.push(target);
        }
    }
    let python = if cfg!(windows) { "python" } else { "python3" };
    let mut command = Command::new(python);
    command.arg("-m").arg("pip").arg("install").arg("--upgrade");
    for package in &packages {
        command.arg(package);
    }
    if !destination.as_os_str().is_empty() {
        command.arg("--target").arg(destination);
    }
    let status = command
        .status()
        .map_err(|e| Error::new("PYTHON001", format!("no se pudo ejecutar Python: {e}")))?;
    if !status.success() {
        return Err(Error::new(
            "PYTHON002",
            "pip no pudo instalar MotorPY y MotorPY-Core",
        ));
    }
    Ok(())
}

fn install_project(client: &GithubClient, version: &Version, destination: &Path) -> Result<()> {
    let url = version
        .source_url
        .as_ref()
        .ok_or_else(|| Error::new("INSTALL003", "la versión no tiene fuente descargable"))?;
    let asset = Asset {
        name: format!("{}.tar.gz", version.tag),
        size: 0,
        url: url.clone(),
        digest: None,
    };
    let path = download_without_size(client, &asset)?;
    std::fs::create_dir_all(destination).map_err(|e| Error::new("INSTALL002", e.to_string()))?;
    extract_tar_gz(&path, destination)?;
    let _ = std::fs::remove_file(path);
    Ok(())
}

fn install_rust_core(client: &GithubClient, version: &Version, destination: &Path) -> Result<()> {
    install_project(client, version, destination)?;
    let manifest = find_manifest(destination)?;
    let manifest_text =
        std::fs::read_to_string(&manifest).map_err(|e| Error::new("RUST001", e.to_string()))?;
    if !manifest_text.contains("pixl-motorru-core") {
        return Err(Error::new(
            "RUST001",
            "la fuente descargada no contiene el crate MotorRU-Core esperado",
        ));
    }
    Ok(())
}

fn install_executable(client: &GithubClient, offer: &Offer, destination: &Path) -> Result<()> {
    let asset = offer
        .assets
        .iter()
        .find(|a| a.name.to_ascii_lowercase().ends_with(".exe"))
        .ok_or_else(|| Error::new("INSTALL003", "no hay un ejecutable compatible"))?;
    let path = download_verified(client, asset)?;
    std::fs::create_dir_all(destination).map_err(|e| Error::new("INSTALL002", e.to_string()))?;
    let target = destination.join(&asset.name);
    if target.exists() {
        return Err(Error::new(
            "INSTALL004",
            format!("ya existe {}", target.display()),
        ));
    }
    std::fs::rename(path, target).map_err(|e| Error::new("INSTALL002", e.to_string()))?;
    Ok(())
}

fn find_manifest(destination: &Path) -> Result<PathBuf> {
    let mut manifests = Vec::new();
    let mut pending = vec![destination.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in
            std::fs::read_dir(&directory).map_err(|e| Error::new("RUST001", e.to_string()))?
        {
            let entry = entry.map_err(|e| Error::new("RUST001", e.to_string()))?;
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.file_name().and_then(|name| name.to_str()) == Some("Cargo.toml") {
                manifests.push(path);
            }
        }
    }
    manifests
        .into_iter()
        .next()
        .ok_or_else(|| Error::new("RUST001", "la fuente descargada no contiene Cargo.toml"))
}

fn download_without_size(client: &GithubClient, asset: &Asset) -> Result<PathBuf> {
    let temp =
        tempfile::NamedTempFile::new().map_err(|e| Error::new("INSTALL001", e.to_string()))?;
    let path = temp.path().to_path_buf();
    client.download(&asset.url, &path)?;
    std::mem::forget(temp);
    Ok(path)
}

fn extract_tar_gz(path: &Path, destination: &Path) -> Result<()> {
    let file = std::fs::File::open(path).map_err(|e| Error::new("EXTRACT001", e.to_string()))?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    for entry in archive
        .entries()
        .map_err(|e| Error::new("EXTRACT001", e.to_string()))?
    {
        let mut entry = entry.map_err(|e| Error::new("EXTRACT001", e.to_string()))?;
        let relative = entry
            .path()
            .map_err(|e| Error::new("EXTRACT002", e.to_string()))?
            .to_path_buf();
        if entry.header().entry_type().is_symlink() || entry.header().entry_type().is_hard_link() {
            return Err(Error::new(
                "EXTRACT002",
                "enlaces no permitidos en el archivo descargado",
            ));
        }
        if relative.is_absolute()
            || relative
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(Error::new(
                "EXTRACT002",
                "ruta insegura en el archivo descargado",
            ));
        }
        let target = destination.join(&relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::new("EXTRACT001", e.to_string()))?;
        }
        entry
            .unpack(&target)
            .map_err(|e| Error::new("EXTRACT001", e.to_string()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hashes_files_in_chunks() {
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), b"pixl").unwrap();
        assert_eq!(
            sha256(file.path()).unwrap(),
            "724772deba58fa070014fb0045126afb893799e05c77a7ca59c279aa272d546c"
        );
    }
}
