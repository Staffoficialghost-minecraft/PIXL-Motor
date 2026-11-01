use crate::diagnostics::{Error, Result};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

const REQUIRED: [&str; 4] = ["motorpy", "motorpy-core", "motorru", "motorru-core"];

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    pub schema_version: u32,
    pub generations: Vec<Generation>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Generation {
    pub id: String,
    pub name: String,
    pub status: String,
    pub components: Vec<Component>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Component {
    pub id: String,
    pub version: String,
    pub distribution: String,
    pub path: String,
    pub format: String,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub platform: Option<String>,
}

impl Manifest {
    pub fn parse(text: &str) -> Result<Self> {
        let manifest: Self = serde_json::from_str(text)
            .map_err(|e| Error::new("MANIFEST001", format!("manifiesto inválido: {e}")))?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1 {
            return Err(Error::new(
                "MANIFEST002",
                "versión de esquema no compatible",
            ));
        }
        for generation in &self.generations {
            generation.validate()?;
        }
        Ok(())
    }

    pub fn latest_complete(&self) -> Option<&Generation> {
        self.generations
            .iter()
            .rev()
            .find(|generation| generation.is_complete() && generation.status == "published")
    }
}

impl Generation {
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty() || self.name.is_empty() {
            return Err(Error::new(
                "MANIFEST003",
                "generación sin identificador o nombre",
            ));
        }
        let mut ids = HashSet::new();
        for component in &self.components {
            if !ids.insert(component.id.as_str()) {
                return Err(Error::new("MANIFEST004", "componente duplicado"));
            }
            if component.path.is_empty()
                || component.path.starts_with('/')
                || component
                    .path
                    .split('/')
                    .any(|part| part == ".." || part.is_empty())
            {
                return Err(Error::new("MANIFEST005", "ruta de componente inválida"));
            }
            if component.version.is_empty() || component.format.is_empty() {
                return Err(Error::new(
                    "MANIFEST006",
                    "componente sin versión o formato",
                ));
            }
            if let Some(hash) = &component.sha256 {
                if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(Error::new("MANIFEST007", "SHA-256 inválido"));
                }
            }
        }
        for component in &self.components {
            for dependency in &component.dependencies {
                if !ids.contains(dependency.as_str()) {
                    return Err(Error::new(
                        "MANIFEST008",
                        format!("dependencia inexistente: {dependency}"),
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        let ids: HashSet<&str> = self.components.iter().map(|c| c.id.as_str()).collect();
        REQUIRED.iter().all(|required| ids.contains(required))
    }

    pub fn components_by_id(&self) -> HashMap<&str, &Component> {
        self.components
            .iter()
            .map(|component| (component.id.as_str(), component))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(status: &str, ids: &[&str]) -> String {
        let components = ids
            .iter()
            .map(|id| {
                format!(
                    r#"{{"id":"{id}","version":"0.1.0","distribution":"source","path":"Code/{id}","format":"source-tree","dependencies":[]}}"#
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            r#"{{"schema_version":1,"generations":[{{"id":"g","name":"G","status":"{status}","components":[{components}]}}]}}"#
        )
    }

    #[test]
    fn accepts_complete_generation_and_selects_latest() {
        let parsed = Manifest::parse(&manifest(
            "published",
            &["motorpy", "motorpy-core", "motorru", "motorru-core"],
        ))
        .unwrap();
        assert_eq!(parsed.latest_complete().unwrap().id, "g");
    }

    #[test]
    fn rejects_incomplete_generation_as_latest() {
        let parsed = Manifest::parse(&manifest("published", &["motorpy"])).unwrap();
        assert!(parsed.latest_complete().is_none());
    }

    #[test]
    fn rejects_unknown_dependency() {
        let text = manifest(
            "published",
            &["motorpy", "motorpy-core", "motorru", "motorru-core"],
        )
        .replace("\"dependencies\":[]", "\"dependencies\":[\"missing\"]");
        assert!(Manifest::parse(&text).is_err());
    }
}
