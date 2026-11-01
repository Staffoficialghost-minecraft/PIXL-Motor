#![cfg_attr(windows, windows_subsystem = "windows")]

use pixl_motorru::{
    Output, process_animation_file, process_file, write_animation_ppm, write_raster_ppm, write_svg,
};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        #[cfg(feature = "gui")]
        {
            if let Err(error) = pixl_motorru::gui::launch(None, 16) {
                eprintln!("{error}");
                std::process::exit(1);
            }
            return;
        }
        #[cfg(not(feature = "gui"))]
        {
            eprintln!("MotorRU fue compilado sin la feature gui");
            std::process::exit(2);
        }
    }
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("Uso:");
        println!("  pixl-motorru archivo.pixl [--output resultado.ppm|resultado.svg]");
        println!("  pixl-motorru --animation archivo.pixl [--output-dir carpeta]");
        println!("  pixl-motorru --gui [archivo.pixl]");
        println!("  pixl-motorru                 (abre la GUI)");
        return;
    }
    if args.len() >= 2 && args[1] == "--gui" && args.len() <= 3 {
        #[cfg(feature = "gui")]
        {
            let path = args.get(2).map(String::as_str);
            if let Err(error) = pixl_motorru::gui::launch(path, 16) {
                eprintln!("{error}");
                std::process::exit(1);
            }
            return;
        }
        #[cfg(not(feature = "gui"))]
        {
            eprintln!("MotorRU fue compilado sin la feature gui");
            std::process::exit(2);
        }
    }
    if args.len() < 2 {
        eprintln!("Use --help para ver la sintaxis");
        std::process::exit(2);
    }
    let animation = args.get(1).is_some_and(|arg| arg == "--animation");
    let path_index = if animation { 1 } else { 0 };
    if args.len() <= path_index + 1 {
        eprintln!("Falta el archivo PIXL");
        std::process::exit(2);
    }
    let path = &args[path_index + 1];
    let output_arg = args
        .windows(2)
        .find(|pair| pair[0] == "--output")
        .map(|pair| pair[1].clone());
    let output_dir = args
        .windows(2)
        .find(|pair| pair[0] == "--output-dir")
        .map(|pair| pair[1].clone());
    for option in ["--output", "--output-dir"] {
        if args.iter().any(|arg| arg == option)
            && args
                .windows(2)
                .find(|pair| pair[0] == option)
                .is_none_or(|pair| pair[1].starts_with("--"))
        {
            eprintln!("Falta el valor de {option}");
            std::process::exit(2);
        }
    }
    if animation && output_arg.is_some() {
        eprintln!("Las animaciones requieren --output-dir");
        std::process::exit(2);
    }
    if !animation && output_dir.is_some() {
        eprintln!("--output-dir solo es válido con --animation");
        std::process::exit(2);
    }
    if args.iter().any(|arg| {
        arg.starts_with("--")
            && !matches!(arg.as_str(), "--animation" | "--output" | "--output-dir")
    }) {
        eprintln!("Argumento inválido");
        std::process::exit(2);
    }
    let result = if animation {
        process_animation_file(path).map_err(|error| error.to_string())
    } else {
        process_file(path).map_err(|error| error.to_string())
    };
    let (output, diagnostics) = match result {
        Ok(value) => value,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    for diagnostic in diagnostics.0 {
        eprintln!(
            "[{:?}] {}:{}:{}:{} {}",
            diagnostic.severity,
            diagnostic.code,
            diagnostic.file,
            diagnostic.line,
            diagnostic.character,
            diagnostic.message
        );
    }
    if output.is_none() {
        std::process::exit(1);
    }
    match output.unwrap() {
        Output::Raster(image) => {
            if let Some(path) = output_arg {
                if let Err(error) = write_raster_ppm(&image, path) {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
            println!(
                "Imagen RGBA {}x{} procesada correctamente",
                image.width, image.height
            );
        }
        Output::Svg(svg) => {
            if let Some(path) = output_arg {
                if let Err(error) = write_svg(&svg, path) {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
            println!("SVG procesado correctamente ({} bytes)", svg.len());
        }
        Output::Animation(animation) => {
            if let Some(directory) = output_dir {
                if let Err(error) = write_animation_ppm(&animation, directory) {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            }
            println!(
                "Animación procesada correctamente: {} frames",
                animation.frames.len()
            );
        }
    }
}
