use crate::diagnostics::{Error, Result};
use crate::distribution::Manifest;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct RefResponse {
    object: RefObject,
}

#[derive(Debug, Deserialize)]
struct RefObject {
    sha: String,
}

pub struct GithubClient {
    client: Client,
    api_url: String,
    owner: String,
    repository: String,
}

impl GithubClient {
    pub fn official() -> Result<Self> {
        Self::new(
            "https://api.github.com",
            "Staffoficialghost-minecraft",
            "PIXL-Motor",
        )
    }

    pub fn new(api_url: &str, owner: &str, repository: &str) -> Result<Self> {
        let client = Client::builder()
            .user_agent("PIXL-Motor-Installer/0.2")
            .build()
            .map_err(|e| Error::new("GITHUB001", e.to_string()))?;
        Ok(Self {
            client,
            api_url: api_url.trim_end_matches('/').to_string(),
            owner: owner.to_string(),
            repository: repository.to_string(),
        })
    }

    pub fn revision(&self) -> Result<String> {
        let url = format!(
            "{}/repos/{}/{}/git/ref/heads/main",
            self.api_url, self.owner, self.repository
        );
        let response = self.client.get(url).send().map_err(|e| {
            Error::new(
                "GITHUB001",
                format!("no se pudo consultar la rama main: {e}"),
            )
        })?;
        if !response.status().is_success() {
            return Err(Error::new(
                "GITHUB001",
                format!("GitHub respondió {} al consultar main", response.status()),
            ));
        }
        response
            .json::<RefResponse>()
            .map(|data| data.object.sha)
            .map_err(|e| Error::new("GITHUB002", format!("respuesta de main inválida: {e}")))
    }

    pub fn manifest(&self, revision: &str) -> Result<Manifest> {
        let url = self.raw_url(revision, "distribution/manifest.json");
        let response = self.client.get(url).send().map_err(|e| {
            Error::new(
                "MANIFEST001",
                format!("no se pudo descargar el manifiesto: {e}"),
            )
        })?;
        if !response.status().is_success() {
            return Err(Error::new(
                "MANIFEST001",
                format!("manifiesto HTTP {}", response.status()),
            ));
        }
        let text = response
            .text()
            .map_err(|e| Error::new("MANIFEST001", e.to_string()))?;
        Manifest::parse(&text)
    }

    pub fn raw_url(&self, revision: &str, path: &str) -> String {
        if self.api_url == "https://api.github.com" {
            format!(
                "https://raw.githubusercontent.com/{}/{}/{}/{}",
                self.owner, self.repository, revision, path
            )
        } else {
            format!(
                "{}/{}/{}/{}/{}",
                self.api_url, self.owner, self.repository, revision, path
            )
        }
    }

    pub fn archive_url(&self, revision: &str) -> String {
        format!(
            "{}/repos/{}/{}/zipball/{}",
            self.api_url, self.owner, self.repository, revision
        )
    }

    pub fn download(&self, url: &str, destination: &Path) -> Result<u64> {
        let mut response = self
            .client
            .get(url)
            .send()
            .map_err(|e| Error::new("DOWNLOAD001", e.to_string()))?;
        if !response.status().is_success() {
            return Err(Error::new(
                "DOWNLOAD001",
                format!("descarga HTTP {}", response.status()),
            ));
        }
        let mut file = std::fs::File::create(destination)
            .map_err(|e| Error::new("DOWNLOAD002", e.to_string()))?;
        response
            .copy_to(&mut file)
            .map_err(|e| Error::new("DOWNLOAD002", e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_main_revision() {
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("GET", "/repos/a/b/git/ref/heads/main")
            .with_status(200)
            .with_body(r#"{"object":{"sha":"abc123"}}"#)
            .create();
        let client = GithubClient::new(&server.url(), "a", "b").unwrap();
        assert_eq!(client.revision().unwrap(), "abc123");
    }

    #[test]
    fn keeps_manifest_and_assets_on_the_same_revision() {
        let client = GithubClient::new("http://mock", "a", "b").unwrap();
        assert_eq!(
            client.raw_url("abc123", "distribution/manifest.json"),
            "http://mock/a/b/abc123/distribution/manifest.json"
        );
        assert_eq!(
            client.archive_url("abc123"),
            "http://mock/repos/a/b/zipball/abc123"
        );
    }
}
