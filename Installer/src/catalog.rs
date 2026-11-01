use crate::versions::{Asset, Version};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Product {
    MotorPyPip,
    MotorPyProject,
    MotorRuCore,
    MotorRuExecutable,
}

impl Product {
    pub fn label(self) -> &'static str {
        match self {
            Self::MotorPyPip => "MotorPY mediante pip",
            Self::MotorPyProject => "MotorPY como proyecto",
            Self::MotorRuCore => "MotorRU-Core como biblioteca Rust",
            Self::MotorRuExecutable => "MotorRU como ejecutable",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Offer {
    pub product: Product,
    pub assets: Vec<Asset>,
}

pub fn catalog(version: &Version) -> Vec<Offer> {
    let py = version
        .assets
        .iter()
        .filter(|a| {
            a.name.starts_with("pixl_motorpy-")
                && (a.name.ends_with(".whl") || a.name.ends_with(".tar.gz"))
        })
        .cloned()
        .collect::<Vec<_>>();
    let core_py = version
        .assets
        .iter()
        .filter(|a| {
            a.name.starts_with("pixl_motorpy_core-")
                && (a.name.ends_with(".whl") || a.name.ends_with(".tar.gz"))
        })
        .cloned()
        .collect::<Vec<_>>();
    let exe = version
        .assets
        .iter()
        .filter(|a| a.name.starts_with("PIXL-MotorRU") && a.name.ends_with(".exe"))
        .cloned()
        .collect::<Vec<_>>();
    let rust = version
        .assets
        .iter()
        .filter(|a| a.name == "pixl-motorru-core.tar.gz" || a.name == "pixl-motorru-core.zip")
        .cloned()
        .collect::<Vec<_>>();
    let mut offers = Vec::new();
    if !py.is_empty() && !core_py.is_empty() {
        offers.push(Offer {
            product: Product::MotorPyPip,
            assets: [py, core_py].concat(),
        });
    }
    if version.source_url.is_some() {
        offers.push(Offer {
            product: Product::MotorPyProject,
            assets: Vec::new(),
        });
    }
    if !rust.is_empty() {
        offers.push(Offer {
            product: Product::MotorRuCore,
            assets: rust,
        });
    }
    if !exe.is_empty() {
        offers.push(Offer {
            product: Product::MotorRuExecutable,
            assets: exe,
        });
    }
    offers
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_complete_pip_offer_is_available() {
        let v = Version {
            name: "v".into(),
            tag: "v".into(),
            description: String::new(),
            published_at: None,
            prerelease: false,
            source_url: None,
            from_release: true,
            assets: vec![Asset {
                name: "pixl_motorpy-0.1-py3-none-any.whl".into(),
                size: 1,
                url: "x".into(),
                digest: None,
            }],
        };
        assert!(catalog(&v).is_empty());
    }
}
