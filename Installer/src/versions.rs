use crate::diagnostics::Result;
use crate::distribution::{Generation, Manifest};
use crate::github::GithubClient;

pub fn refresh(client: &GithubClient) -> Result<(String, Manifest)> {
    let revision = client.revision()?;
    let manifest = client.manifest(&revision)?;
    Ok((revision, manifest))
}

pub fn latest_stable(manifest: &Manifest) -> Option<&Generation> {
    manifest.latest_complete()
}
