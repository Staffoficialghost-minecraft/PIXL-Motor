use pixl_motor_installer::catalog::catalog;
use pixl_motor_installer::diagnostics::{Error, Result};
use pixl_motor_installer::github::GithubClient;
use pixl_motor_installer::installation;
use pixl_motor_installer::versions::{latest_stable, refresh};
use std::io::{self, Write};
use std::path::PathBuf;

fn main() {
    if let Err(error) = run() {
        eprintln!("[ERROR] {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    println!("╔══════════════════════════════════════════╗");
    println!("║            PIXL MOTOR INSTALLER          ║");
    println!("║       Official PIXL Motor Installer      ║");
    println!("╚══════════════════════════════════════════╝");
    let client = GithubClient::official()?;
    let (revision, manifest) = refresh(&client)?;
    let selected = choose_generation(&manifest)?;
    let generation = &manifest.generations[selected];
    println!("\nGeneration: {} ({})", generation.name, generation.id);
    println!("Revision: {}", revision);
    for component in &generation.components {
        println!("  {} {}", component.id, component.version);
    }
    let offers = catalog(generation);
    println!("Select product:");
    for (index, offer) in offers.iter().enumerate() {
        println!("  {}. {}", index + 1, offer.product.label());
    }
    println!("  q. Exit");
    print!("Selection: ");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| Error::new("UI001", e.to_string()))?;
    if input.trim() == "q" {
        return Ok(());
    }
    let number = input
        .trim()
        .parse::<usize>()
        .map_err(|_| Error::new("UI002", "selección inválida"))?;
    let offer = offers
        .get(number.saturating_sub(1))
        .ok_or_else(|| Error::new("UI002", "selección inválida"))?;
    print!("Destination: ");
    io::stdout().flush().ok();
    let mut destination = String::new();
    io::stdin()
        .read_line(&mut destination)
        .map_err(|e| Error::new("UI001", e.to_string()))?;
    installation::install(
        &client,
        &revision,
        offer,
        &PathBuf::from(destination.trim()),
    )?;
    println!("Installation completed.");
    Ok(())
}

fn choose_generation(manifest: &pixl_motor_installer::distribution::Manifest) -> Result<usize> {
    let latest = latest_stable(manifest)
        .ok_or_else(|| Error::new("VERSION001", "no hay generaciones completas publicadas"))?;
    let default = manifest
        .generations
        .iter()
        .position(|generation| generation.id == latest.id)
        .ok_or_else(|| Error::new("MANIFEST009", "Latest no pertenece al manifiesto"))?;
    println!("\nGenerations (Latest = {}):", latest.name);
    for (index, generation) in manifest.generations.iter().enumerate() {
        let marker = if index == default { " [Latest]" } else { "" };
        println!("  {}. {}{}", index + 1, generation.name, marker);
    }
    print!("Generation [Latest]: ");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| Error::new("UI001", e.to_string()))?;
    Ok(input
        .trim()
        .parse::<usize>()
        .ok()
        .and_then(|number| number.checked_sub(1))
        .filter(|index| {
            *index < manifest.generations.len() && manifest.generations[*index].is_complete()
        })
        .unwrap_or(default))
}
