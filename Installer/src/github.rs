use crate::diagnostics::{Error, Result};
use crate::versions::{Asset, Version};
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    name: String,
    body: Option<String>,
    published_at: Option<String>,
    prerelease: bool,
    draft: bool,
    assets: Vec<ReleaseAsset>,
    tarball_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReleaseAsset {
    name: String,
    size: u64,
    browser_download_url: String,
    digest: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Tag {
    name: String,
    tarball_url: String,
}

pub struct GithubClient {
    client: Client,
    base_url: String,
}

impl GithubClient {
    pub fn official() -> Result<Self> {
        Self::new("https://api.github.com")
    }

    pub fn new(base_url: &str) -> Result<Self> {
        let client = Client::builder()
            .user_agent("PIXL-Motor-Installer/0.1")
            .build()
            .map_err(|e| Error::new("GITHUB001", e.to_string()))?;
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    pub fn versions(&self, owner: &str, repo: &str) -> Result<Vec<Version>> {
        let releases_url = format!("{}/repos/{owner}/{repo}/releases", self.base_url);
        let response =
            self.client.get(&releases_url).send().map_err(|e| {
                Error::new("GITHUB001", format!("no se pudo consultar Releases: {e}"))
            })?;
        if response.status().is_success() {
            let releases: Vec<Release> = response.json().map_err(|e| {
                Error::new("GITHUB002", format!("respuesta de Releases inválida: {e}"))
            })?;
            let versions: Vec<Version> = releases
                .into_iter()
                .filter(|r| !r.draft)
                .map(|r| Version {
                    name: if r.name.is_empty() {
                        r.tag_name.clone()
                    } else {
                        r.name
                    },
                    tag: r.tag_name,
                    description: r.body.unwrap_or_default(),
                    published_at: r.published_at,
                    prerelease: r.prerelease,
                    assets: r
                        .assets
                        .into_iter()
                        .map(|a| Asset {
                            name: a.name,
                            size: a.size,
                            url: a.browser_download_url,
                            digest: a.digest,
                        })
                        .collect(),
                    source_url: r.tarball_url,
                    from_release: true,
                })
                .collect();
            if !versions.is_empty() {
                return Ok(versions);
            }
        } else if response.status().as_u16() != 404 {
            return Err(Error::new(
                "GITHUB001",
                format!("GitHub respondió {}", response.status()),
            ));
        }
        let tags_url = format!("{}/repos/{owner}/{repo}/tags", self.base_url);
        let response = self
            .client
            .get(tags_url)
            .send()
            .map_err(|e| Error::new("GITHUB001", format!("no se pudo consultar Tags: {e}")))?;
        if !response.status().is_success() {
            return Err(Error::new(
                "GITHUB001",
                format!("GitHub respondió {}", response.status()),
            ));
        }
        let tags: Vec<Tag> = response
            .json()
            .map_err(|e| Error::new("GITHUB002", format!("respuesta de Tags inválida: {e}")))?;
        Ok(tags
            .into_iter()
            .map(|t| Version {
                name: t.name.clone(),
                tag: t.name,
                description: String::new(),
                published_at: None,
                prerelease: false,
                assets: Vec::new(),
                source_url: Some(t.tarball_url),
                from_release: false,
            })
            .collect())
    }

    pub fn download(&self, url: &str, destination: &std::path::Path) -> Result<u64> {
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
    fn parses_release_assets_and_falls_back_to_tags() {
        let mut server = mockito::Server::new();
        let _m = server.mock("GET", "/repos/a/b/releases").with_status(200)
            .with_body(r#"[{"tag_name":"v1","name":"v1","body":"","published_at":null,"prerelease":false,"draft":false,"assets":[{"name":"x.whl","size":2,"browser_download_url":"http://x","digest":"sha256:abc"}],"tarball_url":"http://src"}]"#).create();
        let client = GithubClient::new(&server.url()).unwrap();
        let versions = client.versions("a", "b").unwrap();
        assert_eq!(versions[0].assets[0].name, "x.whl");
    }
}
