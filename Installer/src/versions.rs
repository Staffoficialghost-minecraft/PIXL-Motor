use crate::diagnostics::Result;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asset {
    pub name: String,
    pub size: u64,
    pub url: String,
    pub digest: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    pub name: String,
    pub tag: String,
    pub description: String,
    pub published_at: Option<String>,
    pub prerelease: bool,
    pub assets: Vec<Asset>,
    pub source_url: Option<String>,
    pub from_release: bool,
}

impl Version {
    pub fn display_name(&self) -> String {
        format!(
            "{}{}",
            self.name,
            if self.prerelease { " (prerelease)" } else { "" }
        )
    }
}

pub fn latest_stable(versions: &[Version]) -> Option<usize> {
    versions
        .iter()
        .enumerate()
        .find(|(_, v)| !v.prerelease && v.from_release)
        .map(|(i, _)| i)
        .or_else(|| {
            versions
                .iter()
                .enumerate()
                .find(|(_, v)| !v.prerelease)
                .map(|(i, _)| i)
        })
}

pub fn refresh(client: &crate::github::GithubClient) -> Result<Vec<Version>> {
    client.versions(crate::OWNER, crate::REPOSITORY)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn v(tag: &str, prerelease: bool, release: bool) -> Version {
        Version {
            name: tag.into(),
            tag: tag.into(),
            description: String::new(),
            published_at: None,
            prerelease,
            assets: Vec::new(),
            source_url: None,
            from_release: release,
        }
    }
    #[test]
    fn latest_prefers_stable_release() {
        assert_eq!(
            latest_stable(&[v("beta", true, true), v("stable", false, true)]),
            Some(1)
        );
    }
}
