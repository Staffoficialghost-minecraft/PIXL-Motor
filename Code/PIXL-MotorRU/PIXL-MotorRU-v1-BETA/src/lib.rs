use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;

#[cfg(feature = "gui")]
pub mod gui;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErrorDefinition {
    pub code: String,
    pub category: String,
    pub description: String,
}

#[derive(Clone, Debug, Default)]
pub struct ErrorRegistry {
    definitions: HashMap<String, ErrorDefinition>,
}

impl ErrorRegistry {
    pub fn core() -> Self {
        let mut registry = Self::default();
        for (code, category, description) in [
            ("PIXL001", "syntax", "Error de sintaxis"),
            ("PIXL010", "version", "Versión PIXL no soportada"),
            ("PIXL030", "coordinates", "Coordenadas inválidas"),
            ("PIXL031", "coordinates", "Rango inválido"),
            ("PIXL032", "color", "Color inválido"),
            ("PIXL038", "geometry", "Vértices repetidos"),
            ("PIXL039", "geometry", "Polígono degenerado"),
            ("PIXL040", "group", "Grupo desconocido"),
            ("PIXL063", "animation", "SVG animado no soportado"),
        ] {
            registry.register(ErrorDefinition {
                code: code.into(),
                category: category.into(),
                description: description.into(),
            });
        }
        registry
    }

    pub fn register(&mut self, definition: ErrorDefinition) -> Option<ErrorDefinition> {
        self.definitions.insert(definition.code.clone(), definition)
    }

    pub fn get(&self, code: &str) -> Option<&ErrorDefinition> {
        self.definitions.get(code)
    }

    pub fn definitions(&self) -> impl Iterator<Item = &ErrorDefinition> {
        self.definitions.values()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub file: String,
    pub line: usize,
    pub character: usize,
    pub message: String,
}

#[derive(Default, Clone, Debug)]
pub struct Diagnostics(pub Vec<Diagnostic>);

impl Diagnostics {
    fn push(
        &mut self,
        severity: Severity,
        code: &str,
        file: &str,
        line: usize,
        character: usize,
        message: impl Into<String>,
    ) {
        self.0.push(Diagnostic {
            severity,
            code: code.into(),
            file: file.into(),
            line,
            character,
            message: message.into(),
        });
    }
    fn error(
        &mut self,
        code: &str,
        file: &str,
        line: usize,
        character: usize,
        message: impl Into<String>,
    ) {
        self.push(Severity::Error, code, file, line, character, message);
    }
    fn warning(
        &mut self,
        code: &str,
        file: &str,
        line: usize,
        character: usize,
        message: impl Into<String>,
    ) {
        self.push(Severity::Warning, code, file, line, character, message);
    }
    pub fn has_errors(&self) -> bool {
        self.0.iter().any(|d| d.severity == Severity::Error)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgba(pub u8, pub u8, pub u8, pub u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PixelState {
    Undeclared,
    Declared,
    Invisible,
}

#[derive(Clone, Copy, Debug)]
enum DrawStyle {
    Fill,
    Outline { thickness: i32, position: char },
}

#[derive(Clone, Debug)]
enum ColorRef {
    Literal(Rgba),
    Components([f64; 4]),
    Name(String),
}

#[derive(Clone, Debug)]
enum Command {
    Pixel(i32, i32, ColorRef),
    Invisible(i32, i32),
    Rect((i32, i32), (i32, i32), ColorRef, DrawStyle),
    Fill((i32, i32), (i32, i32), ColorRef),
    Circle(i32, i32, i32, ColorRef, DrawStyle),
    Ellipse(i32, i32, i32, i32, ColorRef, DrawStyle),
    Polygon(Vec<(i32, i32)>, ColorRef, DrawStyle),
}

#[derive(Clone, Debug, Default)]
struct Layer {
    number: i32,
    groups: HashMap<String, ColorRef>,
    commands: Vec<Command>,
}

#[derive(Clone, Debug, Default)]
struct Frame {
    number: i32,
    duration_ms: i32,
    groups: HashMap<String, ColorRef>,
    layers: Vec<Layer>,
}

#[derive(Clone, Debug, Default)]
struct Document {
    version: Option<String>,
    canvas: Option<(usize, usize)>,
    tipe: Option<String>,
    colors: Option<String>,
    groups: HashMap<String, ColorRef>,
    layers: Vec<Layer>,
    frames: Vec<Frame>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Rgba>,
}

impl Image {
    pub fn pixel(&self, x: usize, y: usize) -> Option<Rgba> {
        if x == 0 || y == 0 || x > self.width || y > self.height {
            return None;
        }
        Some(self.pixels[(y - 1) * self.width + x - 1])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimationFrame {
    pub number: i32,
    pub duration_ms: i32,
    pub image: Image,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Animation {
    pub frames: Vec<AnimationFrame>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Output {
    Raster(Image),
    Svg(String),
    Animation(Animation),
}

#[derive(Debug)]
pub enum ProcessError {
    Io(std::io::Error),
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ProcessError {}

pub fn process_text(text: &str, file: &str) -> (Option<Output>, Diagnostics) {
    let mut diagnostics = Diagnostics::default();
    let document = parse(text, file, &mut diagnostics);
    validate(&document, file, &mut diagnostics);
    if diagnostics.has_errors() {
        return (None, diagnostics);
    }
    if document.version.as_deref() == Some("1.2") {
        diagnostics.error(
            "PIXL054",
            file,
            1,
            1,
            "PIXL(1.2) requiere el procesamiento de animaciones",
        );
        return (None, diagnostics);
    }
    let image = render(&document, &document.layers, file, &mut diagnostics);
    if diagnostics.has_errors() {
        return (None, diagnostics);
    }
    if document.tipe.as_deref() == Some("vec") {
        (Some(Output::Svg(to_svg(&image))), diagnostics)
    } else {
        (Some(Output::Raster(image)), diagnostics)
    }
}

pub fn process_animation(text: &str, file: &str) -> (Option<Output>, Diagnostics) {
    let mut diagnostics = Diagnostics::default();
    let document = parse(text, file, &mut diagnostics);
    validate(&document, file, &mut diagnostics);
    if document.version.as_deref() != Some("1.2") {
        diagnostics.error(
            "PIXL058",
            file,
            1,
            1,
            "process_animation requiere PIXL(1.2)",
        );
    }
    if document.tipe.as_deref() == Some("vec") {
        diagnostics.error(
            "PIXL063",
            file,
            1,
            1,
            "la exportación SVG animada no está implementada",
        );
    }
    if diagnostics.has_errors() {
        return (None, diagnostics);
    }
    let mut frames = Vec::new();
    for frame in &document.frames {
        let mut frame_document = document.clone();
        frame_document.groups.extend(frame.groups.clone());
        let image = render(&frame_document, &frame.layers, file, &mut diagnostics);
        frames.push(AnimationFrame {
            number: frame.number,
            duration_ms: frame.duration_ms,
            image,
        });
    }
    (Some(Output::Animation(Animation { frames })), diagnostics)
}

pub fn process_file(path: impl AsRef<Path>) -> Result<(Option<Output>, Diagnostics), ProcessError> {
    let path = path.as_ref();
    let text = fs::read_to_string(path).map_err(ProcessError::Io)?;
    Ok(process_text(&text, &path.to_string_lossy()))
}

pub fn process_animation_file(
    path: impl AsRef<Path>,
) -> Result<(Option<Output>, Diagnostics), ProcessError> {
    let path = path.as_ref();
    let text = fs::read_to_string(path).map_err(ProcessError::Io)?;
    Ok(process_animation(&text, &path.to_string_lossy()))
}

pub fn process_text_svg(text: &str, file: &str) -> (Option<String>, Diagnostics) {
    let (output, diagnostics) = process_text(text, file);
    match output {
        Some(Output::Svg(svg)) => (Some(svg), diagnostics),
        Some(Output::Raster(_)) | Some(Output::Animation(_)) => (None, diagnostics),
        None => (None, diagnostics),
    }
}

pub fn write_raster_ppm(image: &Image, path: impl AsRef<Path>) -> Result<(), ProcessError> {
    let mut bytes = format!("P6\n{} {}\n255\n", image.width, image.height).into_bytes();
    for pixel in &image.pixels {
        bytes.extend([pixel.0, pixel.1, pixel.2]);
    }
    fs::write(path, bytes).map_err(ProcessError::Io)
}

pub fn write_svg(svg: &str, path: impl AsRef<Path>) -> Result<(), ProcessError> {
    fs::write(path, svg).map_err(ProcessError::Io)
}

pub fn write_animation_ppm(
    animation: &Animation,
    directory: impl AsRef<Path>,
) -> Result<(), ProcessError> {
    fs::create_dir_all(directory.as_ref()).map_err(ProcessError::Io)?;
    for frame in &animation.frames {
        let filename = directory.as_ref().join(format!(
            "frame-{:04}-{}ms.ppm",
            frame.number, frame.duration_ms
        ));
        write_raster_ppm(&frame.image, filename)?;
    }
    Ok(())
}

fn parse(text: &str, file: &str, diagnostics: &mut Diagnostics) -> Document {
    let mut document = Document::default();
    let mut declarations = HashMap::<&str, usize>::new();
    let mut current_layer: Option<Layer> = None;
    let mut current_frame: Option<Frame> = None;
    for (line_index, raw) in text.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let line_no = line_index + 1;
        let character = raw.find(line).map_or(1, |index| index + 1);
        if line.starts_with("GROUP") {
            if let Some((name, color)) = parse_group(line) {
                let target = if let Some(layer) = current_layer.as_mut() {
                    &mut layer.groups
                } else if let Some(frame) = current_frame.as_mut() {
                    &mut frame.groups
                } else {
                    &mut document.groups
                };
                if target.insert(name.clone(), color).is_some() {
                    diagnostics.error("PIXL022", file, line_no, character, "grupo duplicado");
                }
            } else {
                diagnostics.error(
                    "PIXL001",
                    file,
                    line_no,
                    character,
                    "declaración GROUP inválida",
                );
            }
            continue;
        }
        if let Some((name, args)) = call(line) {
            if args.iter().any(|arg| arg.is_empty()) {
                diagnostics.error("PIXL001", file, line_no, character, "argumento vacío");
                continue;
            }
            match name {
                "PIXL" if args.len() == 1 => {
                    duplicate_declaration(&mut declarations, "PIXL", file, line_no, diagnostics);
                    document.version = Some(args[0].to_string());
                }
                "CANVAS" if args.len() == 2 => {
                    duplicate_declaration(&mut declarations, "CANVAS", file, line_no, diagnostics);
                    document.canvas = Some((
                        parse_int(args[0]).max(0) as usize,
                        parse_int(args[1]).max(0) as usize,
                    ))
                }
                "TIPE" if args.len() == 1 => {
                    duplicate_declaration(&mut declarations, "TIPE", file, line_no, diagnostics);
                    document.tipe = Some(args[0].to_string());
                }
                "HEX8" if args.is_empty() => {
                    duplicate_declaration(&mut declarations, "COLOR", file, line_no, diagnostics);
                    document.colors = Some("HEX8".into());
                }
                "RGBA" if args.len() == 1 => {
                    duplicate_declaration(&mut declarations, "COLOR", file, line_no, diagnostics);
                    document.colors = Some(format!("RGBA({})", args[0]));
                }
                "LAYER" if args.len() == 1 => {
                    if let Some(layer) = current_layer.take() {
                        if let Some(frame) = current_frame.as_mut() {
                            frame.layers.push(layer);
                        } else {
                            document.layers.push(layer);
                        }
                    }
                    current_layer = Some(Layer {
                        number: parse_int(args[0]),
                        ..Layer::default()
                    });
                }
                "FRAME" if args.len() == 2 => {
                    if let Some(layer) = current_layer.take() {
                        if let Some(frame) = current_frame.as_mut() {
                            frame.layers.push(layer);
                        }
                    }
                    if let Some(frame) = current_frame.take() {
                        document.frames.push(frame);
                    }
                    current_frame = Some(Frame {
                        number: parse_int(args[0]),
                        duration_ms: parse_int(args[1]),
                        ..Frame::default()
                    });
                }
                "pixel" | "inv" | "rect" | "fill" | "circle" | "ellipse" | "polygon" => {
                    if let Some(command) = parse_command(line, file, line_no, diagnostics) {
                        if let Some(layer) = current_layer.as_mut() {
                            layer.commands.push(command);
                        } else {
                            diagnostics.error(
                                "PIXL001",
                                file,
                                line_no,
                                character,
                                "los comandos deben estar dentro de LAYER",
                            );
                        }
                    }
                }
                _ => diagnostics.error(
                    "PIXL001",
                    file,
                    line_no,
                    character,
                    format!("declaración inválida: {line}"),
                ),
            }
            continue;
        }
        if let Some(command) = parse_command(line, file, line_no, diagnostics) {
            if let Some(layer) = current_layer.as_mut() {
                layer.commands.push(command);
            } else {
                diagnostics.error(
                    "PIXL001",
                    file,
                    line_no,
                    character,
                    "los comandos deben estar dentro de LAYER",
                );
            }
        } else {
            diagnostics.error(
                "PIXL001",
                file,
                line_no,
                character,
                format!("sintaxis inválida: {line}"),
            );
        }
    }
    if let Some(layer) = current_layer.take() {
        if let Some(frame) = current_frame.as_mut() {
            frame.layers.push(layer);
        } else {
            document.layers.push(layer);
        }
    }
    if let Some(frame) = current_frame.take() {
        document.frames.push(frame);
    }
    document
}

fn duplicate_declaration(
    declarations: &mut HashMap<&'static str, usize>,
    name: &'static str,
    file: &str,
    line: usize,
    diagnostics: &mut Diagnostics,
) {
    if declarations.insert(name, line).is_some() {
        diagnostics.error(
            "PIXL021",
            file,
            line,
            1,
            format!("declaración duplicada: {name}"),
        );
    }
}

fn call(line: &str) -> Option<(&str, Vec<&str>)> {
    let open = line.find('(')?;
    if !line.ends_with(')') {
        return None;
    }
    let name = line[..open].trim();
    if name == "GROUP" || name.is_empty() {
        return None;
    }
    let args = split_args(&line[open + 1..line.len() - 1]);
    Some((name, args))
}

fn split_args(value: &str) -> Vec<&str> {
    if value.trim().is_empty() {
        return Vec::new();
    }
    value.split(',').map(str::trim).collect()
}

fn parse_int(value: &str) -> i32 {
    value.parse().unwrap_or(0)
}

fn parse_group(line: &str) -> Option<(String, ColorRef)> {
    let rest = line.strip_prefix("GROUP")?.trim().strip_prefix('=')?.trim();
    let open = rest.find('(')?;
    let name = rest[..open].trim().to_string();
    let args = split_args(rest[open + 1..].strip_suffix(')')?);
    Some((name, parse_color(&args)))
}

fn parse_color(args: &[&str]) -> ColorRef {
    if args.len() == 1 {
        if args[0].len() == 8 && args[0].chars().all(|c| c.is_ascii_hexdigit()) {
            let bytes = (0..4)
                .map(|i| u8::from_str_radix(&args[0][i * 2..i * 2 + 2], 16).unwrap_or(0))
                .collect::<Vec<_>>();
            return ColorRef::Literal(Rgba(bytes[0], bytes[1], bytes[2], bytes[3]));
        }
        return ColorRef::Name(args[0].into());
    }
    if args.len() == 4 {
        let channels = args
            .iter()
            .map(|v| v.parse::<f64>().unwrap_or(f64::NAN))
            .collect::<Vec<_>>();
        return ColorRef::Components([channels[0], channels[1], channels[2], channels[3]]);
    }
    ColorRef::Name(args.first().copied().unwrap_or("").into())
}

fn parse_command(
    line: &str,
    file: &str,
    line_no: usize,
    diagnostics: &mut Diagnostics,
) -> Option<Command> {
    let (name, args) = call(line)?;
    if name == "inv" && args.len() == 2 {
        return Some(Command::Invisible(parse_int(args[0]), parse_int(args[1])));
    }
    let style_position = args.iter().position(|v| *v == "fill" || *v == "outline");
    let geometry_count = match name {
        "pixel" => 2,
        "rect" | "fill" => 2,
        "circle" => 3,
        "ellipse" => 4,
        "polygon" => {
            let style = style_position.unwrap_or(args.len());
            let color_count = if style >= 4
                && args[style - 4..style]
                    .iter()
                    .all(|value| value.parse::<f64>().is_ok())
            {
                4
            } else {
                1
            };
            if style < color_count + 6 || (style - color_count) % 2 != 0 {
                diagnostics.error(
                    "PIXL036",
                    file,
                    line_no,
                    1,
                    "polygon requiere tres vértices",
                );
                return None;
            }
            style - color_count
        }
        _ => {
            diagnostics.error(
                "PIXL001",
                file,
                line_no,
                1,
                format!("comando inválido: {line}"),
            );
            return None;
        }
    };
    if args.len() <= geometry_count {
        diagnostics.error("PIXL032", file, line_no, 1, "color faltante");
        return None;
    }
    let color_count = if args.len() >= geometry_count + 4
        && args[geometry_count..geometry_count + 4]
            .iter()
            .all(|value| value.parse::<f64>().is_ok())
    {
        4
    } else {
        1
    };
    let color_start = geometry_count;
    let color_end = color_start + color_count;
    if color_end > args.len() {
        diagnostics.error("PIXL032", file, line_no, 1, "color inválido");
        return None;
    }
    let color = parse_color(&args[color_start..color_end]);
    let trailing = &args[color_end..];
    if !trailing.is_empty() {
        let valid_style = matches!(trailing.first(), Some(&"fill") | Some(&"outline"))
            && (trailing.len() == 1
                || (trailing.first() == Some(&"outline")
                    && trailing.len() == 3
                    && trailing[1].parse::<i32>().is_ok()
                    && matches!(trailing[2], "i" | "c" | "e")));
        if !valid_style || matches!(name, "pixel" | "fill") {
            diagnostics.error(
                "PIXL035",
                file,
                line_no,
                1,
                "argumentos adicionales inválidos",
            );
            return None;
        }
    }
    let style = parse_style(trailing, file, line_no, diagnostics);
    let range = |v: &str| {
        let parts = v.split('-').map(parse_int).collect::<Vec<_>>();
        if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            (0, 0)
        }
    };
    match name {
        "pixel" => Some(Command::Pixel(
            parse_int(args[0]),
            parse_int(args[1]),
            color,
        )),
        "rect" => Some(Command::Rect(range(args[0]), range(args[1]), color, style)),
        "fill" => Some(Command::Fill(range(args[0]), range(args[1]), color)),
        "circle" => Some(Command::Circle(
            parse_int(args[0]),
            parse_int(args[1]),
            parse_int(args[2]),
            color,
            style,
        )),
        "ellipse" => Some(Command::Ellipse(
            parse_int(args[0]),
            parse_int(args[1]),
            parse_int(args[2]),
            parse_int(args[3]),
            color,
            style,
        )),
        "polygon" => {
            let points = args[..geometry_count]
                .chunks_exact(2)
                .map(|pair| (parse_int(pair[0]), parse_int(pair[1])))
                .collect();
            Some(Command::Polygon(points, color, style))
        }
        _ => None,
    }
}

fn parse_style(
    trailing: &[&str],
    file: &str,
    line: usize,
    diagnostics: &mut Diagnostics,
) -> DrawStyle {
    if trailing.is_empty() || trailing[0] == "fill" {
        return DrawStyle::Fill;
    }
    if trailing[0] != "outline" {
        return DrawStyle::Fill;
    }
    let thickness = if trailing.len() >= 2 {
        parse_int(trailing[1])
    } else {
        1
    };
    let position = if trailing.len() >= 3 {
        trailing[2].chars().next().unwrap_or('c')
    } else {
        'c'
    };
    if thickness <= 0 || !matches!(position, 'i' | 'c' | 'e') {
        diagnostics.error(
            "PIXL035",
            file,
            line,
            1,
            "outline requiere grosor positivo y posición i, c o e",
        );
        return DrawStyle::Fill;
    }
    DrawStyle::Outline {
        thickness,
        position,
    }
}

fn validate(document: &Document, file: &str, diagnostics: &mut Diagnostics) {
    if !matches!(document.version.as_deref(), Some("1.1") | Some("1.2")) {
        diagnostics.error("PIXL010", file, 1, 1, "versión PIXL no soportada");
    }
    if document.canvas.is_none() || document.canvas.is_some_and(|(w, h)| w == 0 || h == 0) {
        diagnostics.error("PIXL012", file, 1, 1, "dimensiones inválidas");
    }
    if !matches!(document.tipe.as_deref(), Some("bit") | Some("vec")) {
        diagnostics.error("PIXL013", file, 1, 1, "TIPE debe ser bit o vec");
    }
    if document.colors.is_none() {
        diagnostics.error("PIXL014", file, 1, 1, "falta sistema de color");
    } else if !matches!(
        document.colors.as_deref(),
        Some("HEX8") | Some("RGBA(1)") | Some("RGBA(2)")
    ) {
        diagnostics.error("PIXL014", file, 1, 1, "sistema de color no soportado");
    }
    if document.version.as_deref() == Some("1.2") {
        if document.frames.is_empty() {
            diagnostics.error("PIXL056", file, 1, 1, "PIXL(1.2) requiere frames");
        }
        if document.tipe.as_deref() == Some("vec") {
            diagnostics.error("PIXL063", file, 1, 1, "SVG animado no soportado");
        }
        let mut previous = 0;
        for frame in &document.frames {
            if frame.number <= previous || frame.number <= 0 {
                diagnostics.error("PIXL050", file, 1, 1, "numeración de frame inválida");
            }
            if frame.duration_ms <= 0 || frame.layers.is_empty() {
                diagnostics.error("PIXL053", file, 1, 1, "frame inválido");
            }
            validate_layers(document, &frame.layers, file, diagnostics);
            validate_groups(&frame.groups, document, file, diagnostics);
            for layer in &frame.layers {
                validate_groups(&layer.groups, document, file, diagnostics);
            }
            validate_layer_numbers(&frame.layers, file, diagnostics);
            previous = frame.number;
        }
    } else if !document.frames.is_empty() {
        diagnostics.error(
            "PIXL057",
            file,
            1,
            1,
            "FRAME solo está disponible en PIXL(1.2)",
        );
    }
    if document.version.as_deref() == Some("1.1") && document.layers.is_empty() {
        diagnostics.error("PIXL015", file, 1, 1, "falta LAYER");
    }
    validate_groups(&document.groups, document, file, diagnostics);
    for layer in &document.layers {
        validate_groups(&layer.groups, document, file, diagnostics);
    }
    validate_layers(document, &document.layers, file, diagnostics);
    validate_layer_numbers(&document.layers, file, diagnostics);
}

fn validate_groups(
    groups: &HashMap<String, ColorRef>,
    document: &Document,
    file: &str,
    diagnostics: &mut Diagnostics,
) {
    for color in groups.values() {
        match color {
            ColorRef::Name(_) => {
                diagnostics.error("PIXL033", file, 1, 1, "GROUP requiere un color literal")
            }
            ColorRef::Components(values) => {
                let valid = match document.colors.as_deref() {
                    Some("RGBA(1)") => values.iter().all(|value| (0.0..=1.0).contains(value)),
                    Some("RGBA(2)") => values.iter().all(|value| (0.0..=255.0).contains(value)),
                    _ => false,
                };
                if !valid {
                    diagnostics.error("PIXL033", file, 1, 1, "color inválido en GROUP");
                }
            }
            ColorRef::Literal(_) => {
                if document.colors.as_deref() != Some("HEX8") {
                    diagnostics.error("PIXL033", file, 1, 1, "HEX8 no corresponde al sistema RGBA");
                }
            }
        }
    }
}

fn validate_layer_numbers(layers: &[Layer], file: &str, diagnostics: &mut Diagnostics) {
    let mut seen = HashMap::new();
    for layer in layers {
        if layer.number <= 0 {
            diagnostics.error("PIXL016", file, 1, 1, "el número de capa debe ser positivo");
        }
        if seen.insert(layer.number, ()).is_some() {
            diagnostics.error("PIXL017", file, 1, 1, "capa duplicada");
        }
    }
}

fn validate_layers(
    document: &Document,
    layers: &[Layer],
    file: &str,
    diagnostics: &mut Diagnostics,
) {
    let (width, height) = document.canvas.unwrap_or((0, 0));
    for layer in layers {
        for command in &layer.commands {
            match command {
                Command::Pixel(x, y, _) | Command::Invisible(x, y) => {
                    if !in_canvas(*x, *y, width, height) {
                        diagnostics.error("PIXL030", file, 1, 1, "coordenadas fuera de rango");
                    }
                }
                Command::Rect((x1, x2), (y1, y2), _, _) | Command::Fill((x1, x2), (y1, y2), _) => {
                    if *x1 < 1
                        || *y1 < 1
                        || x1 > x2
                        || y1 > y2
                        || *x2 as usize > width
                        || *y2 as usize > height
                    {
                        diagnostics.error("PIXL031", file, 1, 1, "rango de coordenadas inválido");
                    }
                }
                Command::Circle(_, _, radius, _, _) if *radius <= 0 => {
                    diagnostics.error("PIXL037", file, 1, 1, "radio inválido")
                }
                Command::Ellipse(x1, y1, x2, y2, _, _) if x1 >= x2 || y1 >= y2 => {
                    diagnostics.error("PIXL037", file, 1, 1, "límites de elipse inválidos")
                }
                Command::Polygon(points, _, _) => {
                    if points.windows(2).any(|pair| pair[0] == pair[1])
                        || points.first() == points.last()
                    {
                        diagnostics.error("PIXL038", file, 1, 1, "vértices repetidos");
                    }
                    let area: i64 = points
                        .iter()
                        .zip(points.iter().cycle().skip(1))
                        .take(points.len())
                        .map(|(a, b)| a.0 as i64 * b.1 as i64 - b.0 as i64 * a.1 as i64)
                        .sum();
                    if area == 0 {
                        diagnostics.error("PIXL039", file, 1, 1, "polígono degenerado");
                    }
                }
                _ => {}
            }
        }
    }
}

fn render(
    document: &Document,
    layers: &[Layer],
    file: &str,
    diagnostics: &mut Diagnostics,
) -> Image {
    let (width, height) = document.canvas.unwrap_or((0, 0));
    let mut final_pixels = vec![Rgba(0, 0, 0, 0); width * height];
    for layer in layers {
        let mut pixels = vec![Rgba(0, 0, 0, 0); width * height];
        let mut states = vec![PixelState::Undeclared; width * height];
        for command in &layer.commands {
            draw_command(
                document,
                layer,
                command,
                &mut pixels,
                &mut states,
                width,
                height,
                file,
                diagnostics,
            );
        }
        for (dst, src) in final_pixels.iter_mut().zip(pixels) {
            *dst = over(*dst, src);
        }
    }
    Image {
        width,
        height,
        pixels: final_pixels,
    }
}

fn draw_command(
    document: &Document,
    layer: &Layer,
    command: &Command,
    pixels: &mut [Rgba],
    states: &mut [PixelState],
    width: usize,
    height: usize,
    file: &str,
    diagnostics: &mut Diagnostics,
) {
    let color = match command {
        Command::Pixel(_, _, c)
        | Command::Rect(_, _, c, _)
        | Command::Fill(_, _, c)
        | Command::Circle(_, _, _, c, _)
        | Command::Ellipse(_, _, _, _, c, _)
        | Command::Polygon(_, c, _) => resolve_color(c, document, layer, file, diagnostics),
        Command::Invisible(_, _) => None,
    };
    match command {
        Command::Invisible(x, y) => set_invisible(*x, *y, states, pixels, width, height),
        Command::Pixel(x, y, _) => {
            if let Some(c) = color {
                set_pixel(*x, *y, c, states, pixels, width, height);
            }
        }
        Command::Rect(xr, yr, _, style) => {
            for y in 1..=height as i32 {
                for x in 1..=width as i32 {
                    let selected = match style {
                        DrawStyle::Fill => x >= xr.0 && x <= xr.1 && y >= yr.0 && y <= yr.1,
                        DrawStyle::Outline {
                            thickness,
                            position,
                        } => {
                            let distance = [
                                (x - xr.0).abs(),
                                (x - xr.1).abs(),
                                (y - yr.0).abs(),
                                (y - yr.1).abs(),
                            ]
                            .into_iter()
                            .min()
                            .unwrap_or(i32::MAX);
                            match position {
                                'i' => {
                                    x >= xr.0
                                        && x <= xr.1
                                        && y >= yr.0
                                        && y <= yr.1
                                        && distance < *thickness
                                }
                                'e' => {
                                    (x < xr.0 || x > xr.1 || y < yr.0 || y > yr.1)
                                        && distance <= *thickness
                                }
                                _ => distance * 2 <= *thickness,
                            }
                        }
                    };
                    if selected {
                        if let Some(c) = color {
                            set_pixel(x, y, c, states, pixels, width, height);
                        }
                    }
                }
            }
        }
        Command::Fill(xr, yr, _) => {
            if let Some(c) = color {
                for y in yr.0..=yr.1 {
                    for x in xr.0..=xr.1 {
                        if in_canvas(x, y, width, height)
                            && states[(y as usize - 1) * width + x as usize - 1]
                                == PixelState::Undeclared
                        {
                            set_pixel(x, y, c, states, pixels, width, height);
                        }
                    }
                }
            }
        }
        Command::Circle(cx, cy, radius, _, style) => {
            if let Some(c) = color {
                for y in 1..=height as i32 {
                    for x in 1..=width as i32 {
                        let d = ((x as f64 - 0.5 - *cx as f64).powi(2)
                            + (y as f64 - 0.5 - *cy as f64).powi(2))
                        .sqrt();
                        let selected = match style {
                            DrawStyle::Fill => d <= *radius as f64,
                            DrawStyle::Outline {
                                thickness,
                                position,
                            } => {
                                let distance = (d - *radius as f64).abs();
                                match position {
                                    'i' => {
                                        d <= *radius as f64
                                            && *radius as f64 - d <= *thickness as f64
                                    }
                                    'e' => d >= *radius as f64 && distance <= *thickness as f64,
                                    _ => distance <= *thickness as f64 / 2.0,
                                }
                            }
                        };
                        if selected {
                            set_pixel(x, y, c, states, pixels, width, height);
                        }
                    }
                }
            }
        }
        Command::Ellipse(x1, y1, x2, y2, _, style) => {
            if let Some(c) = color {
                let cx = (*x1 + *x2) as f64 / 2.0;
                let cy = (*y1 + *y2) as f64 / 2.0;
                let rx = (*x2 - *x1).abs() as f64 / 2.0;
                let ry = (*y2 - *y1).abs() as f64 / 2.0;
                if rx > 0.0 && ry > 0.0 {
                    for y in 1..=height as i32 {
                        for x in 1..=width as i32 {
                            let v = ((x as f64 - 0.5 - cx) / rx).powi(2)
                                + ((y as f64 - 0.5 - cy) / ry).powi(2);
                            let selected = match style {
                                DrawStyle::Fill => v <= 1.0,
                                DrawStyle::Outline {
                                    thickness,
                                    position,
                                } => {
                                    let distance = (v.sqrt() - 1.0).abs() * rx.min(ry);
                                    match position {
                                        'i' => v <= 1.0 && distance <= *thickness as f64,
                                        'e' => v >= 1.0 && distance <= *thickness as f64,
                                        _ => distance <= *thickness as f64 / 2.0,
                                    }
                                }
                            };
                            if selected {
                                set_pixel(x, y, c, states, pixels, width, height);
                            }
                        }
                    }
                }
            }
        }
        Command::Polygon(points, _, style) => {
            if let Some(c) = color {
                for y in 1..=height as i32 {
                    for x in 1..=width as i32 {
                        let px = x as f64 - 0.5;
                        let py = y as f64 - 0.5;
                        let inside = point_in_polygon(px, py, points);
                        let selected = match style {
                            DrawStyle::Fill => inside,
                            DrawStyle::Outline {
                                thickness,
                                position,
                            } => {
                                let distance = edge_distance(px, py, points);
                                match position {
                                    'i' => inside && distance <= *thickness as f64,
                                    'e' => !inside && distance <= *thickness as f64,
                                    _ => distance <= *thickness as f64 / 2.0,
                                }
                            }
                        };
                        if selected {
                            set_pixel(x, y, c, states, pixels, width, height);
                        }
                    }
                }
            }
        }
    }
}

fn resolve_color(
    color: &ColorRef,
    document: &Document,
    layer: &Layer,
    file: &str,
    diagnostics: &mut Diagnostics,
) -> Option<Rgba> {
    match color {
        ColorRef::Literal(value) => Some(*value),
        ColorRef::Components(values) => {
            let (valid, scale) = match document.colors.as_deref() {
                Some("RGBA(1)") => (
                    values.iter().all(|value| (0.0..=1.0).contains(value)),
                    255.0,
                ),
                Some("RGBA(2)") => (
                    values.iter().all(|value| (0.0..=255.0).contains(value)),
                    1.0,
                ),
                _ => (false, 1.0),
            };
            if !valid {
                diagnostics.error(
                    "PIXL032",
                    file,
                    1,
                    1,
                    "color inválido para el sistema declarado",
                );
                return None;
            }
            Some(Rgba(
                (values[0] * scale).round() as u8,
                (values[1] * scale).round() as u8,
                (values[2] * scale).round() as u8,
                (values[3] * scale).round() as u8,
            ))
        }
        ColorRef::Name(name) => {
            let value = layer.groups.get(name).or_else(|| document.groups.get(name));
            if let Some(value) = value {
                return resolve_color(value, document, layer, file, diagnostics);
            }
            diagnostics.warning("PIXL040", file, 1, 1, format!("grupo desconocido: {name}"));
            Some(Rgba(0, 0, 0, 0))
        }
    }
}

fn in_canvas(x: i32, y: i32, width: usize, height: usize) -> bool {
    x > 0 && y > 0 && (x as usize) <= width && (y as usize) <= height
}
fn set_pixel(
    x: i32,
    y: i32,
    color: Rgba,
    states: &mut [PixelState],
    pixels: &mut [Rgba],
    width: usize,
    height: usize,
) {
    if in_canvas(x, y, width, height) {
        let i = (y as usize - 1) * width + x as usize - 1;
        states[i] = PixelState::Declared;
        pixels[i] = color;
    }
}
fn set_invisible(
    x: i32,
    y: i32,
    states: &mut [PixelState],
    pixels: &mut [Rgba],
    width: usize,
    height: usize,
) {
    if in_canvas(x, y, width, height) {
        let i = (y as usize - 1) * width + x as usize - 1;
        states[i] = PixelState::Invisible;
        pixels[i] = Rgba(0, 0, 0, 0);
    }
}
fn over(background: Rgba, foreground: Rgba) -> Rgba {
    let sa = foreground.3 as f64 / 255.;
    let da = background.3 as f64 / 255.;
    let oa = sa + da * (1. - sa);
    if oa == 0. {
        return Rgba(0, 0, 0, 0);
    }
    Rgba(
        ((foreground.0 as f64 * sa + background.0 as f64 * da * (1. - sa)) / oa).round() as u8,
        ((foreground.1 as f64 * sa + background.1 as f64 * da * (1. - sa)) / oa).round() as u8,
        ((foreground.2 as f64 * sa + background.2 as f64 * da * (1. - sa)) / oa).round() as u8,
        (oa * 255.).round() as u8,
    )
}
fn point_in_polygon(x: f64, y: f64, points: &[(i32, i32)]) -> bool {
    let mut inside = false;
    for (a, b) in points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
    {
        if ((a.1 as f64 > y) != (b.1 as f64 > y))
            && x < (b.0 - a.0) as f64 * (y - a.1 as f64) / (b.1 - a.1) as f64 + a.0 as f64
        {
            inside = !inside;
        }
    }
    inside
}
fn edge_distance(x: f64, y: f64, points: &[(i32, i32)]) -> f64 {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .map(|(a, b)| {
            let dx = (b.0 - a.0) as f64;
            let dy = (b.1 - a.1) as f64;
            let t = (((x - a.0 as f64) * dx + (y - a.1 as f64) * dy) / (dx * dx + dy * dy))
                .clamp(0., 1.);
            ((x - (a.0 as f64 + t * dx)).powi(2) + (y - (a.1 as f64 + t * dy)).powi(2)).sqrt()
        })
        .fold(f64::INFINITY, f64::min)
}
fn to_svg(image: &Image) -> String {
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
        image.width, image.height, image.width, image.height
    );
    for y in 1..=image.height {
        for x in 1..=image.width {
            let p = image.pixel(x, y).unwrap();
            if p.3 > 0 {
                svg.push_str(&format!(r##"<rect x="{}" y="{}" width="1" height="1" fill="#{:02X}{:02X}{:02X}" fill-opacity="{}"/>"##, x - 1, y - 1, p.0, p.1, p.2, p.3 as f64 / 255.0));
            }
        }
    }
    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn renders_pixel() {
        let (output, diagnostics) = process_text(
            "PIXL(1.1)\nCANVAS(1, 1)\nTIPE(bit)\nHEX8()\nLAYER(1)\npixel(1, 1, FF0000FF)\n",
            "test.pixl",
        );
        assert!(!diagnostics.has_errors(), "{:?}", diagnostics.0);
        assert_eq!(
            output,
            Some(Output::Raster(Image {
                width: 1,
                height: 1,
                pixels: vec![Rgba(255, 0, 0, 255)]
            }))
        );
    }
}
