use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub character: usize,
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
    pub category: String,
    pub file: String,
    pub line: usize,
    pub character: usize,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Diagnostics(pub Vec<Diagnostic>);
impl Diagnostics {
    fn add(
        &mut self,
        severity: Severity,
        code: &str,
        category: &str,
        file: &str,
        line: usize,
        character: usize,
        message: impl Into<String>,
    ) {
        self.0.push(Diagnostic {
            severity,
            code: code.into(),
            category: category.into(),
            file: file.into(),
            line,
            character,
            message: message.into(),
        });
    }
    fn error(
        &mut self,
        code: &str,
        category: &str,
        file: &str,
        loc: SourceLocation,
        msg: impl Into<String>,
    ) {
        self.add(
            Severity::Error,
            code,
            category,
            file,
            loc.line,
            loc.character,
            msg,
        );
    }
    fn warning(
        &mut self,
        code: &str,
        category: &str,
        file: &str,
        loc: SourceLocation,
        msg: impl Into<String>,
    ) {
        self.add(
            Severity::Warning,
            code,
            category,
            file,
            loc.line,
            loc.character,
            msg,
        );
    }
    pub fn has_errors(&self) -> bool {
        self.0.iter().any(|d| d.severity == Severity::Error)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorRGBA8 {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Canvas {
    pub width: usize,
    pub height: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutlinePosition {
    Interior,
    Center,
    Exterior,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Fill,
    Outline {
        thickness: i32,
        position: OutlinePosition,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    Pixel {
        x: i32,
        y: i32,
        color: ColorRGBA8,
        location: SourceLocation,
    },
    Invisible {
        x: i32,
        y: i32,
        location: SourceLocation,
    },
    Rect {
        x_range: (i32, i32),
        y_range: (i32, i32),
        color: ColorRGBA8,
        style: Style,
        location: SourceLocation,
    },
    Fill {
        x_range: (i32, i32),
        y_range: (i32, i32),
        color: ColorRGBA8,
        write_policy: &'static str,
        location: SourceLocation,
    },
    Circle {
        x: i32,
        y: i32,
        radius: i32,
        color: ColorRGBA8,
        style: Style,
        location: SourceLocation,
    },
    Ellipse {
        x_range: (i32, i32),
        y_range: (i32, i32),
        color: ColorRGBA8,
        style: Style,
        location: SourceLocation,
    },
    Polygon {
        points: Vec<(i32, i32)>,
        color: ColorRGBA8,
        style: Style,
        location: SourceLocation,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessedLayer {
    pub number: i32,
    pub operations: Vec<Operation>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessedFrame {
    pub number: i32,
    pub duration_ms: i32,
    pub layers: Vec<ProcessedLayer>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessedDocument {
    pub language_version: String,
    pub document_type: String,
    pub color_system: String,
    pub canvas: Canvas,
    pub layers: Vec<ProcessedLayer>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessedAnimation {
    pub language_version: String,
    pub document_type: String,
    pub color_system: String,
    pub canvas: Canvas,
    pub frames: Vec<ProcessedFrame>,
}

#[derive(Clone, Debug)]
enum ColorRef {
    Literal(String),
    Components([f64; 4]),
    Name(String),
}
#[derive(Clone, Debug)]
enum Command {
    Pixel(i32, i32, ColorRef, SourceLocation),
    Invisible(i32, i32, SourceLocation),
    Rect((i32, i32), (i32, i32), ColorRef, Style, SourceLocation),
    Fill((i32, i32), (i32, i32), ColorRef, SourceLocation),
    Circle(i32, i32, i32, ColorRef, Style, SourceLocation),
    Ellipse(i32, i32, i32, i32, ColorRef, Style, SourceLocation),
    Polygon(Vec<(i32, i32)>, ColorRef, Style, SourceLocation),
}
#[derive(Clone, Debug, Default)]
struct Layer {
    number: i32,
    groups: HashMap<String, ColorRef>,
    commands: Vec<Command>,
    location: SourceLocation,
}
#[derive(Clone, Debug, Default)]
struct Frame {
    number: i32,
    duration_ms: i32,
    groups: HashMap<String, ColorRef>,
    layers: Vec<Layer>,
    location: SourceLocation,
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

pub fn process_text(text: &str, file: &str) -> (Option<ProcessedDocument>, Diagnostics) {
    let mut diagnostics = Diagnostics::default();
    let document = parse(text, file, &mut diagnostics);
    validate(&document, file, &mut diagnostics);
    if document.version.as_deref() == Some("1.2") {
        diagnostics.error(
            "PIXL054",
            "animation",
            file,
            SourceLocation {
                line: 1,
                character: 1,
            },
            "PIXL(1.2) requiere process_animation",
        );
    }
    if diagnostics.has_errors() {
        return (None, diagnostics);
    }
    (
        Some(normalize_document(&document, file, &mut diagnostics)),
        diagnostics,
    )
}

pub fn process_animation(text: &str, file: &str) -> (Option<ProcessedAnimation>, Diagnostics) {
    let mut diagnostics = Diagnostics::default();
    let document = parse(text, file, &mut diagnostics);
    validate(&document, file, &mut diagnostics);
    if document.version.as_deref() != Some("1.2") {
        diagnostics.error(
            "PIXL058",
            "animation",
            file,
            SourceLocation {
                line: 1,
                character: 1,
            },
            "process_animation requiere PIXL(1.2)",
        );
    }
    if diagnostics.has_errors() {
        return (None, diagnostics);
    }
    (
        Some(normalize_animation(&document, file, &mut diagnostics)),
        diagnostics,
    )
}

pub fn process_file(
    path: impl AsRef<Path>,
) -> Result<(Option<ProcessedDocument>, Diagnostics), ProcessError> {
    let path = path.as_ref();
    let source = fs::read_to_string(path).map_err(ProcessError::Io)?;
    Ok(process_text(&source, &path.to_string_lossy()))
}
pub fn process_animation_file(
    path: impl AsRef<Path>,
) -> Result<(Option<ProcessedAnimation>, Diagnostics), ProcessError> {
    let path = path.as_ref();
    let source = fs::read_to_string(path).map_err(ProcessError::Io)?;
    Ok(process_animation(&source, &path.to_string_lossy()))
}

fn parse(text: &str, file: &str, diagnostics: &mut Diagnostics) -> Document {
    let mut document = Document::default();
    let mut declarations = HashMap::<&str, ()>::new();
    let mut current_layer: Option<Layer> = None;
    let mut current_frame: Option<Frame> = None;
    for (index, raw) in text.lines().enumerate() {
        let line_no = index + 1;
        let source = raw.split('#').next().unwrap_or("").trim();
        if source.is_empty() {
            continue;
        }
        let loc = SourceLocation {
            line: line_no,
            character: raw.find(source).unwrap_or(0) + 1,
        };
        if source.starts_with("GROUP") {
            if let Some((name, value)) = parse_group(source) {
                let target = if let Some(layer) = current_layer.as_mut() {
                    &mut layer.groups
                } else if let Some(frame) = current_frame.as_mut() {
                    &mut frame.groups
                } else {
                    &mut document.groups
                };
                if target.insert(name, value).is_some() {
                    diagnostics.error("PIXL022", "group", file, loc, "grupo duplicado");
                }
            } else {
                diagnostics.error("PIXL001", "syntax", file, loc, "declaración GROUP inválida");
            }
            continue;
        }
        let Some((name, args)) = call(source) else {
            diagnostics.error("PIXL001", "syntax", file, loc, "sintaxis inválida");
            continue;
        };
        if ["PIXL", "CANVAS", "TIPE", "HEX8", "RGBA"].contains(&name) {
            let key = if matches!(name, "HEX8" | "RGBA") {
                "COLOR"
            } else {
                name
            };
            if !declarations.insert(key, ()).is_none() {
                diagnostics.error(
                    "PIXL021",
                    "syntax",
                    file,
                    loc,
                    format!("declaración duplicada: {name}"),
                );
            }
        }
        match name {
            "PIXL" if args.len() == 1 => document.version = Some(args[0].to_owned()),
            "CANVAS" if args.len() == 2 => {
                document.canvas = Some((
                    parse_i(args[0]).max(0) as usize,
                    parse_i(args[1]).max(0) as usize,
                ))
            }
            "TIPE" if args.len() == 1 => document.tipe = Some(args[0].to_owned()),
            "HEX8" if args.is_empty() => document.colors = Some("HEX8".into()),
            "RGBA" if args.len() == 1 => document.colors = Some(format!("RGBA({})", args[0])),
            "FRAME" if args.len() == 2 => {
                finish_layer(&mut current_layer, &mut current_frame, &mut document);
                if let Some(frame) = current_frame.take() {
                    document.frames.push(frame);
                }
                current_frame = Some(Frame {
                    number: parse_i(args[0]),
                    duration_ms: parse_i(args[1]),
                    location: loc,
                    ..Frame::default()
                });
            }
            "LAYER" if args.len() == 1 => {
                finish_layer(&mut current_layer, &mut current_frame, &mut document);
                current_layer = Some(Layer {
                    number: parse_i(args[0]),
                    location: loc,
                    ..Layer::default()
                });
            }
            "pixel" | "inv" | "rect" | "fill" | "circle" | "ellipse" | "polygon" => {
                match parse_command(name, &args, loc, document.colors.as_deref()) {
                    Some(command) => {
                        if let Some(layer) = current_layer.as_mut() {
                            layer.commands.push(command);
                        }
                    }
                    None => {
                        diagnostics.error("PIXL001", "syntax", file, loc, "argumentos inválidos")
                    }
                }
            }
            _ => diagnostics.error("PIXL001", "syntax", file, loc, "declaración inválida"),
        }
    }
    finish_layer(&mut current_layer, &mut current_frame, &mut document);
    if let Some(frame) = current_frame.take() {
        document.frames.push(frame);
    }
    document
}

fn finish_layer(layer: &mut Option<Layer>, frame: &mut Option<Frame>, document: &mut Document) {
    if let Some(value) = layer.take() {
        if let Some(parent) = frame.as_mut() {
            parent.layers.push(value);
        } else {
            document.layers.push(value);
        }
    }
}
fn call(line: &str) -> Option<(&str, Vec<&str>)> {
    let open = line.find('(')?;
    if !line.ends_with(')') {
        return None;
    }
    Some((
        line[..open].trim(),
        split_args(&line[open + 1..line.len() - 1]),
    ))
}
fn split_args(value: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    for (index, c) in value.char_indices() {
        if c == ',' {
            result.push(value[start..index].trim());
            start = index + 1;
        }
    }
    if start < value.len() {
        result.push(value[start..].trim());
    }
    result
}
fn parse_group(line: &str) -> Option<(String, ColorRef)> {
    let rest = line.strip_prefix("GROUP")?.trim().strip_prefix('=')?.trim();
    let (name, args) = call(rest)?;
    if args.len() == 1 {
        Some((name.into(), ColorRef::Literal(args[0].into())))
    } else if args.len() == 4 {
        Some((
            name.into(),
            ColorRef::Components([
                parse_f(args[0]),
                parse_f(args[1]),
                parse_f(args[2]),
                parse_f(args[3]),
            ]),
        ))
    } else {
        None
    }
}
fn parse_i(value: &str) -> i32 {
    value.parse().unwrap_or(0)
}
fn parse_f(value: &str) -> f64 {
    value.parse().unwrap_or(f64::NAN)
}
fn range(value: &str) -> Option<(i32, i32)> {
    let mut parts = value.split('-');
    Some((parse_i(parts.next()?), parse_i(parts.next()?)))
}
fn style(args: &[&str]) -> Option<Style> {
    if args.is_empty() || args[0] == "fill" {
        return (args.len() == 1 || args.is_empty()).then_some(Style::Fill);
    }
    if args.first()? != &"outline" || (args.len() != 1 && args.len() != 3) {
        return None;
    }
    let thickness = if args.len() == 3 { parse_i(args[1]) } else { 1 };
    let position = match args.get(2).copied().unwrap_or("c") {
        "i" => OutlinePosition::Interior,
        "e" => OutlinePosition::Exterior,
        "c" => OutlinePosition::Center,
        _ => return None,
    };
    Some(Style::Outline {
        thickness,
        position,
    })
}
fn color(args: &[&str], system: Option<&str>) -> Option<ColorRef> {
    match system? {
        "HEX8"
            if args.len() == 1
                && args[0].len() == 8
                && args[0].chars().all(|c| c.is_ascii_hexdigit()) =>
        {
            Some(ColorRef::Literal(args[0].into()))
        }
        "RGBA(1)" | "RGBA(2)" if args.len() == 4 => Some(ColorRef::Components([
            parse_f(args[0]),
            parse_f(args[1]),
            parse_f(args[2]),
            parse_f(args[3]),
        ])),
        _ => None,
    }
}
fn parse_command(
    name: &str,
    args: &[&str],
    loc: SourceLocation,
    system: Option<&str>,
) -> Option<Command> {
    let name_color = |args: &[&str]| -> Option<(ColorRef, usize)> {
        let count = if system == Some("HEX8") { 1 } else { 4 };
        if args.len() < count {
            return None;
        }
        if let Some(value) = color(&args[args.len() - count..], system) {
            return Some((value, args.len() - count));
        }
        if count == 1 {
            Some((ColorRef::Name(args[args.len() - 1].into()), args.len() - 1))
        } else {
            None
        }
    };
    match name {
        "inv" if args.len() == 2 => {
            Some(Command::Invisible(parse_i(args[0]), parse_i(args[1]), loc))
        }
        "pixel" => {
            let (c, i) = name_color(args)?;
            (i == 2).then_some(Command::Pixel(parse_i(args[0]), parse_i(args[1]), c, loc))
        }
        "rect" | "fill" => {
            if args.len() < 3 {
                return None;
            }
            let xr = range(args[0])?;
            let yr = range(args[1])?;
            let (c, i) = name_color(&args[2..])?;
            if i != 1 && i != 0 {
                return None;
            }
            if name == "fill" {
                (args.len() == 3).then_some(Command::Fill(xr, yr, c, loc))
            } else {
                style(&args[3 + i..]).map(|s| Command::Rect(xr, yr, c, s, loc))
            }
        }
        "circle" => {
            let (c, i) = name_color(&args[3..])?;
            (i == 0).then_some(Command::Circle(
                parse_i(args[0]),
                parse_i(args[1]),
                parse_i(args[2]),
                c,
                style(&args[4 + i..])?,
                loc,
            ))
        }
        "ellipse" => {
            let (c, i) = name_color(&args[4..])?;
            (i == 0).then_some(Command::Ellipse(
                parse_i(args[0]),
                parse_i(args[1]),
                parse_i(args[2]),
                parse_i(args[3]),
                c,
                style(&args[5 + i..])?,
                loc,
            ))
        }
        "polygon" => {
            let component_count = if system == Some("HEX8") { 1 } else { 4 };
            let style_start = if args.last() == Some(&"fill") {
                args.len() - 1
            } else if args.len() >= 3 && args[args.len() - 3] == "outline" {
                args.len() - 3
            } else {
                args.len()
            };
            if style_start < component_count {
                return None;
            }
            let color_index = style_start - component_count;
            let c = color(&args[color_index..style_start], system).or_else(|| {
                (component_count == 1).then(|| ColorRef::Name(args[color_index].into()))
            })?;
            let i = color_index;
            if i < 6 || i % 2 != 0 {
                return None;
            }
            let points = args[..i]
                .chunks_exact(2)
                .map(|p| (parse_i(p[0]), parse_i(p[1])))
                .collect();
            Some(Command::Polygon(
                points,
                c,
                style(&args[style_start..])?,
                loc,
            ))
        }
        _ => None,
    }
}

fn validate(doc: &Document, file: &str, d: &mut Diagnostics) {
    let at = SourceLocation {
        line: 1,
        character: 1,
    };
    if !matches!(doc.version.as_deref(), Some("1.1") | Some("1.2")) {
        d.error("PIXL010", "version", file, at, "versión PIXL no soportada");
    }
    if doc.canvas.is_none() || doc.canvas.is_some_and(|(w, h)| w == 0 || h == 0) {
        d.error("PIXL012", "coordinates", file, at, "dimensiones inválidas");
    }
    if !matches!(doc.tipe.as_deref(), Some("bit") | Some("vec")) {
        d.error("PIXL013", "configuration", file, at, "TIPE inválido");
    }
    if !matches!(
        doc.colors.as_deref(),
        Some("HEX8") | Some("RGBA(1)") | Some("RGBA(2)")
    ) {
        d.error("PIXL014", "color", file, at, "sistema de color inválido");
    }
    if doc.version.as_deref() == Some("1.2") {
        if !doc.layers.is_empty() {
            d.error("PIXL055", "animation", file, at, "capas fuera de FRAME");
        }
        if doc.frames.is_empty() {
            d.error(
                "PIXL056",
                "animation",
                file,
                at,
                "PIXL(1.2) requiere frames",
            );
        }
        let mut previous = 0;
        for frame in &doc.frames {
            if frame.number <= previous || frame.number <= 0 {
                d.error(
                    "PIXL050",
                    "animation",
                    file,
                    frame.location,
                    "numeración de frame inválida",
                );
            }
            if frame.duration_ms <= 0 || frame.layers.is_empty() {
                d.error(
                    "PIXL053",
                    "animation",
                    file,
                    frame.location,
                    "frame inválido",
                );
            }
            validate_layers(doc, &frame.layers, file, d);
            previous = frame.number;
        }
    } else {
        if !doc.frames.is_empty() {
            d.error(
                "PIXL057",
                "animation",
                file,
                at,
                "FRAME solo está disponible en PIXL(1.2)",
            );
        }
        if doc.layers.is_empty() {
            d.error("PIXL015", "structure", file, at, "falta LAYER");
        }
        validate_layers(doc, &doc.layers, file, d);
    }
}
fn validate_layers(doc: &Document, layers: &[Layer], file: &str, d: &mut Diagnostics) {
    let (w, h) = doc.canvas.unwrap_or((0, 0));
    let mut seen = HashMap::new();
    for layer in layers {
        if layer.number <= 0 {
            d.error(
                "PIXL016",
                "structure",
                file,
                layer.location,
                "número de capa inválido",
            );
        }
        if seen.insert(layer.number, ()).is_some() {
            d.error(
                "PIXL017",
                "structure",
                file,
                layer.location,
                "capa duplicada",
            );
        }
        for command in &layer.commands {
            match command {
                Command::Pixel(x, y, _, loc) | Command::Invisible(x, y, loc)
                    if !inside(*x, *y, w, h) =>
                {
                    d.error(
                        "PIXL030",
                        "coordinates",
                        file,
                        *loc,
                        "coordenadas fuera de rango",
                    )
                }
                Command::Rect(xr, yr, _, _, loc) | Command::Fill(xr, yr, _, loc)
                    if xr.0 < 1
                        || yr.0 < 1
                        || xr.0 > xr.1
                        || yr.0 > yr.1
                        || xr.1 as usize > w
                        || yr.1 as usize > h =>
                {
                    d.error("PIXL031", "coordinates", file, *loc, "rango inválido")
                }
                Command::Circle(_, _, r, _, _, loc) if *r <= 0 => {
                    d.error("PIXL037", "geometry", file, *loc, "radio inválido")
                }
                Command::Ellipse(x1, y1, x2, y2, _, _, loc) if x1 >= x2 || y1 >= y2 => {
                    d.error("PIXL037", "geometry", file, *loc, "elipse inválida")
                }
                Command::Polygon(points, _, _, loc) => {
                    if points.len() < 3 {
                        d.error("PIXL036", "geometry", file, *loc, "polygon inválido");
                    }
                    if points.windows(2).any(|p| p[0] == p[1]) || points.first() == points.last() {
                        d.error("PIXL038", "geometry", file, *loc, "vértices repetidos");
                    }
                    let area: i64 = points
                        .iter()
                        .zip(points.iter().cycle().skip(1))
                        .take(points.len())
                        .map(|(a, b)| a.0 as i64 * b.1 as i64 - b.0 as i64 * a.1 as i64)
                        .sum();
                    if area == 0 {
                        d.error("PIXL039", "geometry", file, *loc, "polígono degenerado");
                    }
                }
                _ => {}
            }
        }
    }
}
fn inside(x: i32, y: i32, w: usize, h: usize) -> bool {
    x >= 1 && y >= 1 && (x as usize) <= w && (y as usize) <= h
}

fn normalize_document(doc: &Document, file: &str, d: &mut Diagnostics) -> ProcessedDocument {
    let global = doc.groups.clone();
    let layers = doc
        .layers
        .iter()
        .map(|l| normalize_layer(l, doc, &global, file, d))
        .collect();
    ProcessedDocument {
        language_version: doc.version.clone().unwrap_or_default(),
        document_type: doc.tipe.clone().unwrap_or_default(),
        color_system: doc.colors.clone().unwrap_or_default(),
        canvas: Canvas {
            width: doc.canvas.unwrap_or((0, 0)).0,
            height: doc.canvas.unwrap_or((0, 0)).1,
        },
        layers,
    }
}
fn normalize_animation(doc: &Document, file: &str, d: &mut Diagnostics) -> ProcessedAnimation {
    let canvas = doc.canvas.unwrap_or((0, 0));
    let frames = doc
        .frames
        .iter()
        .map(|f| {
            let mut groups = doc.groups.clone();
            groups.extend(f.groups.clone());
            ProcessedFrame {
                number: f.number,
                duration_ms: f.duration_ms,
                layers: f
                    .layers
                    .iter()
                    .map(|l| normalize_layer(l, doc, &groups, file, d))
                    .collect(),
            }
        })
        .collect();
    ProcessedAnimation {
        language_version: doc.version.clone().unwrap_or_default(),
        document_type: doc.tipe.clone().unwrap_or_default(),
        color_system: doc.colors.clone().unwrap_or_default(),
        canvas: Canvas {
            width: canvas.0,
            height: canvas.1,
        },
        frames,
    }
}
fn normalize_layer(
    layer: &Layer,
    doc: &Document,
    parent: &HashMap<String, ColorRef>,
    file: &str,
    d: &mut Diagnostics,
) -> ProcessedLayer {
    let mut groups = parent.clone();
    groups.extend(layer.groups.clone());
    let operations = layer
        .commands
        .iter()
        .filter_map(|c| normalize_command(c, doc, &groups, file, d))
        .collect();
    ProcessedLayer {
        number: layer.number,
        operations,
    }
}
fn resolve(
    c: &ColorRef,
    doc: &Document,
    groups: &HashMap<String, ColorRef>,
    file: &str,
    d: &mut Diagnostics,
    loc: SourceLocation,
) -> Option<ColorRGBA8> {
    let value = match c {
        ColorRef::Name(name) => groups.get(name),
        _ => Some(c),
    };
    let Some(value) = value else {
        d.warning(
            "PIXL040",
            "group",
            file,
            loc,
            "grupo desconocido; se usará transparencia",
        );
        return Some(ColorRGBA8 {
            red: 0,
            green: 0,
            blue: 0,
            alpha: 0,
        });
    };
    match value {
        ColorRef::Literal(hex) if doc.colors.as_deref() == Some("HEX8") && hex.len() == 8 => {
            let bytes = (0..4)
                .map(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok())
                .collect::<Option<Vec<_>>>();
            bytes
                .map(|b| ColorRGBA8 {
                    red: b[0],
                    green: b[1],
                    blue: b[2],
                    alpha: b[3],
                })
                .or_else(|| {
                    d.error("PIXL032", "color", file, loc, "color inválido");
                    None
                })
        }
        ColorRef::Components(v) => {
            let scale = if doc.colors.as_deref() == Some("RGBA(1)") {
                255.0
            } else {
                1.0
            };
            if (doc.colors.as_deref() == Some("RGBA(1)")
                && v.iter().any(|x| !(0.0..=1.0).contains(x)))
                || (doc.colors.as_deref() == Some("RGBA(2)")
                    && v.iter().any(|x| !(0.0..=255.0).contains(x)))
            {
                d.error("PIXL032", "color", file, loc, "color inválido");
                return None;
            }
            Some(ColorRGBA8 {
                red: (v[0] * scale).round() as u8,
                green: (v[1] * scale).round() as u8,
                blue: (v[2] * scale).round() as u8,
                alpha: (v[3] * scale).round() as u8,
            })
        }
        _ => {
            d.error("PIXL032", "color", file, loc, "color inválido");
            None
        }
    }
}
fn normalize_command(
    c: &Command,
    doc: &Document,
    groups: &HashMap<String, ColorRef>,
    file: &str,
    d: &mut Diagnostics,
) -> Option<Operation> {
    match c {
        Command::Pixel(x, y, c, l) => Some(Operation::Pixel {
            x: *x,
            y: *y,
            color: resolve(c, doc, groups, file, d, *l)?,
            location: *l,
        }),
        Command::Invisible(x, y, l) => Some(Operation::Invisible {
            x: *x,
            y: *y,
            location: *l,
        }),
        Command::Rect(xr, yr, c, s, l) => Some(Operation::Rect {
            x_range: *xr,
            y_range: *yr,
            color: resolve(c, doc, groups, file, d, *l)?,
            style: *s,
            location: *l,
        }),
        Command::Fill(xr, yr, c, l) => Some(Operation::Fill {
            x_range: *xr,
            y_range: *yr,
            color: resolve(c, doc, groups, file, d, *l)?,
            write_policy: "UNDECLARED_ONLY",
            location: *l,
        }),
        Command::Circle(x, y, r, c, s, l) => Some(Operation::Circle {
            x: *x,
            y: *y,
            radius: *r,
            color: resolve(c, doc, groups, file, d, *l)?,
            style: *s,
            location: *l,
        }),
        Command::Ellipse(x1, y1, x2, y2, c, s, l) => Some(Operation::Ellipse {
            x_range: (*x1, *x2),
            y_range: (*y1, *y2),
            color: resolve(c, doc, groups, file, d, *l)?,
            style: *s,
            location: *l,
        }),
        Command::Polygon(points, c, s, l) => Some(Operation::Polygon {
            points: points.clone(),
            color: resolve(c, doc, groups, file, d, *l)?,
            style: *s,
            location: *l,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalizes_pixel_and_rgba() {
        let (result, diagnostics) = process_text(
            "PIXL(1.1)\nCANVAS(1,1)\nTIPE(bit)\nRGBA(1)\nLAYER(1)\npixel(1,1,1,0,0,1)\n",
            "test.pixl",
        );
        assert!(!diagnostics.has_errors());
        assert!(matches!(
            result.unwrap().layers[0].operations[0],
            Operation::Pixel {
                color: ColorRGBA8 {
                    red: 255,
                    green: 0,
                    blue: 0,
                    alpha: 255
                },
                ..
            }
        ));
    }
    #[test]
    fn fill_policy_and_unknown_group() {
        let (result, diagnostics) = process_text(
            "PIXL(1.1)\nCANVAS(1,1)\nTIPE(bit)\nHEX8()\nLAYER(1)\npixel(1,1,missing)\nfill(1-1,1-1,FFFFFFFF)\n",
            "test.pixl",
        );
        assert!(result.is_some());
        assert!(diagnostics.0.iter().any(|d| d.code == "PIXL040"));
        assert!(matches!(
            result.unwrap().layers[0].operations[1],
            Operation::Fill {
                write_policy: "UNDECLARED_ONLY",
                ..
            }
        ));
    }
}
