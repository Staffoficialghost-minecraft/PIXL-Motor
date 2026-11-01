use crate::distribution::{Component, Generation};

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
    pub components: Vec<Component>,
}

pub fn catalog(generation: &Generation) -> Vec<Offer> {
    let by_id = generation.components_by_id();
    let mut offers = Vec::new();
    if let (Some(py), Some(core)) = (by_id.get("motorpy"), by_id.get("motorpy-core")) {
        if py.dependencies.iter().any(|id| id == "motorpy-core") {
            offers.push(Offer {
                product: Product::MotorPyPip,
                components: vec![(*core).clone(), (*py).clone()],
            });
        }
    }
    if let Some(py) = by_id.get("motorpy") {
        offers.push(Offer {
            product: Product::MotorPyProject,
            components: vec![(*py).clone()],
        });
    }
    if let Some(core) = by_id.get("motorru-core") {
        offers.push(Offer {
            product: Product::MotorRuCore,
            components: vec![(*core).clone()],
        });
    }
    if let Some(motorru) = by_id.get("motorru") {
        offers.push(Offer {
            product: Product::MotorRuExecutable,
            components: vec![(*motorru).clone()],
        });
    }
    offers
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distribution::Manifest;

    #[test]
    fn complete_generation_exposes_all_modalities() {
        let text = include_str!("../../distribution/manifest.json");
        let manifest = Manifest::parse(text).unwrap();
        assert_eq!(catalog(manifest.latest_complete().unwrap()).len(), 4);
    }
}
