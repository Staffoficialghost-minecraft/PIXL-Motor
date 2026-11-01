use crate::catalog::{Offer, Product};
use crate::diagnostics::{Error, Result};
use crate::distribution::Component;
use crate::github::GithubClient;
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn install(
    client: &GithubClient,
    revision: &str,
    offer: &Offer,
    destination: &Path,
) -> Result<()> {
    validate_offer(offer)?;
    match offer.product {
        Product::MotorPyPip => install_python(client, revision, offer, destination),
        Product::MotorPyProject => {
            install_source(client, revision, &offer.components[0], destination)
        }
        Product::MotorRuCore => install_source(client, revision, &offer.components[0], destination),
        Product::MotorRuExecutable => {
            install_executable(client, revision, &offer.components[0], destination)
        }
    }
}

fn validate_offer(offer: &Offer) -> Result<()> {
    if offer.components.is_empty() {
        return Err(Error::new("INSTALL001", "modalidad sin componentes"));
    }
    for component in &offer.components {
        if component.path.contains("..") || component.path.starts_with('/') {
            return Err(Error::new("MANIFEST005", "ruta de componente insegura"));
        }
        if component.format.is_empty() {
            return Err(Error::new("MANIFEST006", "formato de componente vacío"));
        }
    }
    Ok(())
}

fn temporary_path(name: &str) -> Result<(tempfile::TempDir, PathBuf)> {
    let directory = tempfile::tempdir().map_err(|e| Error::new("INSTALL001", e.to_string()))?;
    let path = directory.path().join(name);
    Ok((directory, path))
}

fn download_component(
    client: &GithubClient,
    revision: &str,
    component: &Component,
) -> Result<(tempfile::TempDir, PathBuf)> {
    let (directory, path) = temporary_path(
        Path::new(&component.path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("component"),
    )?;
    client.download(&client.raw_url(revision, &component.path), &path)?;
    verify_hash(&path, component.sha256.as_deref())?;
    Ok((directory, path))
}

fn install_python(
    client: &GithubClient,
    revision: &str,
    offer: &Offer,
    destination: &Path,
) -> Result<()> {
    let temp = tempfile::tempdir().map_err(|e| Error::new("INSTALL001", e.to_string()))?;
    let python = if cfg!(windows) { "python" } else { "python3" };
    for component in &offer.components {
        if component.format != "source-tree" {
            return Err(Error::new(
                "PYTHON001",
                "MotorPY requiere fuentes source-tree",
            ));
        }
        let component_dir = temp.path().join(&component.id);
        extract_component_tree(client, revision, component, &component_dir)?;
        let mut command = Command::new(python);
        command.args(["-m", "pip", "install", "--upgrade"]);
        if !destination.as_os_str().is_empty() {
            command.arg("--target").arg(destination);
        }
        command.arg(&component_dir);
        let status = command
            .status()
            .map_err(|e| Error::new("PYTHON001", format!("no se pudo ejecutar Python: {e}")))?;
        if !status.success() {
            return Err(Error::new(
                "PYTHON002",
                "pip no pudo instalar el componente",
            ));
        }
    }
    Ok(())
}

fn install_source(
    client: &GithubClient,
    revision: &str,
    component: &Component,
    destination: &Path,
) -> Result<()> {
    if component.format != "source-tree" {
        return Err(Error::new(
            "INSTALL003",
            "el componente no es una fuente Cargo/Python",
        ));
    }
    let target = destination.join(
        Path::new(&component.path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(component.id.as_str()),
    );
    if target.exists() {
        return Err(Error::new(
            "INSTALL004",
            format!("ya existe {}", target.display()),
        ));
    }
    extract_component_tree(client, revision, component, destination)
}

fn install_executable(
    client: &GithubClient,
    revision: &str,
    component: &Component,
    destination: &Path,
) -> Result<()> {
    if component.format != "exe" {
        return Err(Error::new(
            "INSTALL003",
            "el componente no es un ejecutable",
        ));
    }
    let (_temp, path) = download_component(client, revision, component)?;
    std::fs::create_dir_all(destination).map_err(|e| Error::new("INSTALL002", e.to_string()))?;
    let target = destination.join(
        Path::new(&component.path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("pixl-motorru.exe"),
    );
    if target.exists() {
        return Err(Error::new(
            "INSTALL004",
            format!("ya existe {}", target.display()),
        ));
    }
    std::fs::copy(path, target).map_err(|e| Error::new("INSTALL002", e.to_string()))?;
    Ok(())
}

fn extract_component_tree(
    client: &GithubClient,
    revision: &str,
    component: &Component,
    destination: &Path,
) -> Result<()> {
    let (_temp, archive_path) = temporary_path("source.zip")?;
    client.download(&client.archive_url(revision), &archive_path)?;
    let bytes =
        std::fs::read(&archive_path).map_err(|e| Error::new("EXTRACT001", e.to_string()))?;
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| Error::new("EXTRACT001", e.to_string()))?;
    let prefix = format!("{}/", component.path.trim_end_matches('/'));
    let mut found = false;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|e| Error::new("EXTRACT001", e.to_string()))?;
        let name = entry.name().replace('\\', "/");
        let marker = name.find('/');
        let relative = match marker {
            Some(marker) => &name[marker + 1..],
            None => continue,
        };
        if !relative.starts_with(&prefix) {
            continue;
        }
        let output_relative = &relative[prefix.len()..];
        if output_relative.is_empty() || output_relative.contains("..") {
            return Err(Error::new(
                "EXTRACT002",
                "ruta insegura en el archivo descargado",
            ));
        }
        found = true;
        let target = destination.join(output_relative);
        if entry.is_dir() {
            std::fs::create_dir_all(&target)
                .map_err(|e| Error::new("EXTRACT001", e.to_string()))?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| Error::new("EXTRACT001", e.to_string()))?;
            }
            let mut output = std::fs::File::create(&target)
                .map_err(|e| Error::new("EXTRACT001", e.to_string()))?;
            std::io::copy(&mut entry, &mut output)
                .map_err(|e| Error::new("EXTRACT001", e.to_string()))?;
        }
    }
    if !found {
        return Err(Error::new(
            "EXTRACT003",
            format!("ruta inexistente en la revisión: {}", component.path),
        ));
    }
    Ok(())
}

fn verify_hash(path: &Path, expected: Option<&str>) -> Result<()> {
    let Some(expected) = expected else {
        return Ok(());
    };
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
    let actual = format!("{:x}", hasher.finalize());
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(Error::new("VERIFY002", "SHA-256 no coincide"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_hash() {
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), b"pixl").unwrap();
        assert!(verify_hash(file.path(), Some(&"0".repeat(64))).is_err());
    }
}
