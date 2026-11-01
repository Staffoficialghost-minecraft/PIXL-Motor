use pixl_motor_installer::catalog::catalog;
use pixl_motor_installer::diagnostics::Result;
use pixl_motor_installer::github::GithubClient;
use pixl_motor_installer::installation;
use pixl_motor_installer::versions::{Version, latest_stable, refresh};
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
    println!("Repository: Staffoficialghost-minecraft/PIXL-Motor");
    let client = GithubClient::official()?;
    let mut versions = refresh(&client)?;
    loop {
        let selected = choose_version(&versions)?;
        let version = &versions[selected];
        println!("\nVersion: {}", version.display_name());
        let offers = catalog(version);
        println!("Select product:");
        for (index, offer) in offers.iter().enumerate() {
            println!("  {}. {}", index + 1, offer.product.label());
        }
        println!("  r. Refresh versions");
        println!("  q. Exit");
        print!("Selection: ");
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| pixl_motor_installer::diagnostics::Error::new("UI001", e.to_string()))?;
        match input.trim() {
            "r" => versions = refresh(&client)?,
            "q" => return Ok(()),
            value => {
                if let Ok(number) = value.parse::<usize>() {
                    if let Some(offer) = offers.get(number.saturating_sub(1)) {
                        print!("Destination (blank for current Python environment): ");
                        io::stdout().flush().ok();
                        let mut destination = String::new();
                        io::stdin().read_line(&mut destination).map_err(|e| {
                            pixl_motor_installer::diagnostics::Error::new("UI001", e.to_string())
                        })?;
                        let destination = PathBuf::from(destination.trim());
                        installation::install(&client, version, offer, &destination)?;
                        println!("Installation completed.");
                    } else {
                        println!("Invalid selection.");
                    }
                }
            }
        }
    }
}

fn choose_version(versions: &[Version]) -> Result<usize> {
    let default = latest_stable(versions).ok_or_else(|| {
        pixl_motor_installer::diagnostics::Error::new("VERSION001", "no hay versiones instalables")
    })?;
    println!(
        "\nVersions (Latest = {}):",
        versions[default].display_name()
    );
    for (index, version) in versions.iter().enumerate() {
        println!("  {}. {}", index + 1, version.display_name());
    }
    print!("Version [Latest]: ");
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| pixl_motor_installer::diagnostics::Error::new("UI001", e.to_string()))?;
    Ok(input
        .trim()
        .parse::<usize>()
        .ok()
        .and_then(|n| n.checked_sub(1))
        .filter(|n| *n < versions.len())
        .unwrap_or(default))
}
