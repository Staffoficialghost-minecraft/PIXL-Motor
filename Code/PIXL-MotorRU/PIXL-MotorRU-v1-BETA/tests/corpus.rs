use pixl_motorru::{
    Output, Rgba, process_animation, process_text, write_animation_ppm, write_raster_ppm, write_svg,
};

const CORPUS: &str = "../../PIXL-MotorPY/PIXL-MotorPY-v1-BETA/tests/corpus/";

fn corpus(path: &str) -> String {
    std::fs::read_to_string(format!("{CORPUS}{path}")).expect("neutral corpus case")
}

#[test]
fn basic_raster_matches_rgba_reference() {
    let (output, diagnostics) = process_text(&corpus("valid/basic.pixl"), "basic.pixl");
    assert!(!diagnostics.has_errors(), "{:?}", diagnostics.0);
    match output {
        Some(Output::Raster(image)) => assert_eq!(
            image.pixels,
            vec![Rgba(255, 0, 0, 255), Rgba(0, 255, 0, 255)]
        ),
        other => panic!("unexpected output: {other:?}"),
    }
}

#[test]
fn svg_case_is_semantically_nonempty() {
    let (output, diagnostics) = process_text(&corpus("svg/vector.pixl"), "vector.pixl");
    assert!(!diagnostics.has_errors(), "{:?}", diagnostics.0);
    match output {
        Some(Output::Svg(svg)) => {
            assert!(svg.contains("width=\"4\""));
            assert!(svg.contains("height=\"2\""));
            assert!(svg.contains("fill=\"#112233\""));
        }
        other => panic!("unexpected output: {other:?}"),
    }
}

#[test]
fn animation_case_preserves_frame_metadata() {
    let (output, diagnostics) = process_animation(&corpus("animation/basic.pixl"), "basic.pixl");
    assert!(!diagnostics.has_errors(), "{:?}", diagnostics.0);
    match output {
        Some(Output::Animation(animation)) => {
            assert_eq!(animation.frames.len(), 2);
            assert_eq!(
                (animation.frames[0].number, animation.frames[0].duration_ms),
                (1, 10)
            );
            assert_eq!(
                (animation.frames[1].number, animation.frames[1].duration_ms),
                (2, 20)
            );
        }
        other => panic!("unexpected output: {other:?}"),
    }
}

#[test]
fn invalid_geometry_is_blocking() {
    for (path, code) in [
        ("geometry/degenerate.pixl", "PIXL039"),
        ("geometry/repeated-vertex.pixl", "PIXL038"),
    ] {
        let (output, diagnostics) = process_text(&corpus(path), path);
        assert!(output.is_none());
        assert!(
            diagnostics
                .0
                .iter()
                .any(|diagnostic| diagnostic.code == code),
            "{path}: {:?}",
            diagnostics.0
        );
    }
}

#[test]
fn polygon_rgba_arguments_do_not_become_vertices() {
    let text = "PIXL(1.1)\nCANVAS(4, 4)\nTIPE(bit)\nRGBA(2)\nLAYER(1)\npolygon(1, 1, 4, 1, 1, 4, 255, 0, 0, 255, fill)\n";
    let (output, diagnostics) = process_text(text, "polygon.pixl");
    assert!(!diagnostics.has_errors(), "{:?}", diagnostics.0);
    assert!(matches!(output, Some(Output::Raster(_))));
}

#[test]
fn duplicate_layers_are_rejected() {
    let text = "PIXL(1.1)\nCANVAS(1, 1)\nTIPE(bit)\nHEX8()\nLAYER(1)\npixel(1, 1, FFFFFFFF)\nLAYER(1)\npixel(1, 1, FFFFFFFF)\n";
    let (output, diagnostics) = process_text(text, "duplicate.pixl");
    assert!(output.is_none());
    assert!(
        diagnostics
            .0
            .iter()
            .any(|diagnostic| diagnostic.code == "PIXL017")
    );
}

#[test]
fn color_systems_produce_the_same_rgba() {
    let inputs = [
        ("HEX8()", "FF000080"),
        ("RGBA(2)", "255, 0, 0, 128"),
        ("RGBA(1)", "1, 0, 0, 0.5019607843137255"),
    ];
    let mut pixels = Vec::new();
    for (system, color) in inputs {
        let text = format!(
            "PIXL(1.1)\nCANVAS(1, 1)\nTIPE(bit)\n{system}\nLAYER(1)\npixel(1, 1, {color})\n"
        );
        let (output, diagnostics) = process_text(&text, "colors.pixl");
        assert!(!diagnostics.has_errors(), "{:?}", diagnostics.0);
        match output {
            Some(Output::Raster(image)) => pixels.push(image.pixels[0]),
            other => panic!("unexpected output: {other:?}"),
        }
    }
    assert_eq!(pixels, vec![Rgba(255, 0, 0, 128); 3]);
}

#[test]
fn fill_preserves_invisible_and_declared_pixels() {
    let text = "PIXL(1.1)\nCANVAS(3, 1)\nTIPE(bit)\nHEX8()\nLAYER(1)\npixel(1, 1, FF0000FF)\ninv(2, 1)\nfill(1-3, 1-1, 00FF00FF)\n";
    let (output, diagnostics) = process_text(text, "fill.pixl");
    assert!(!diagnostics.has_errors(), "{:?}", diagnostics.0);
    match output {
        Some(Output::Raster(image)) => assert_eq!(
            image.pixels,
            vec![Rgba(255, 0, 0, 255), Rgba(0, 0, 0, 0), Rgba(0, 255, 0, 255)]
        ),
        other => panic!("unexpected output: {other:?}"),
    }
}

#[test]
fn groups_use_layer_then_frame_then_global_scope() {
    let text = "PIXL(1.2)\nCANVAS(1, 1)\nTIPE(bit)\nHEX8()\nGROUP = color(FF0000FF)\nFRAME(1, 10)\nGROUP = color(00FF00FF)\nLAYER(1)\nGROUP = color(0000FFFF)\npixel(1, 1, color)\n";
    let (output, diagnostics) = process_animation(text, "groups.pixl");
    assert!(!diagnostics.has_errors(), "{:?}", diagnostics.0);
    match output {
        Some(Output::Animation(animation)) => {
            assert_eq!(animation.frames[0].image.pixels, vec![Rgba(0, 0, 255, 255)]);
        }
        other => panic!("unexpected output: {other:?}"),
    }
}

#[test]
fn extra_pixel_arguments_are_rejected() {
    let text =
        "PIXL(1.1)\nCANVAS(1, 1)\nTIPE(bit)\nHEX8()\nLAYER(1)\npixel(1, 1, FFFFFFFF, extra)\n";
    let (output, diagnostics) = process_text(text, "extra.pixl");
    assert!(output.is_none());
    assert!(
        diagnostics
            .0
            .iter()
            .any(|diagnostic| diagnostic.code == "PIXL035")
    );
}

#[test]
fn outline_accepts_thickness_and_position() {
    let text = "PIXL(1.1)\nCANVAS(3, 3)\nTIPE(bit)\nHEX8()\nLAYER(1)\nrect(1-2, 1-2, FFFFFFFF, outline, 1, c)\n";
    let (output, diagnostics) = process_text(text, "outline.pixl");
    assert!(!diagnostics.has_errors(), "{:?}", diagnostics.0);
    match output {
        Some(Output::Raster(image)) => {
            assert_eq!(image.pixels[0], Rgba(255, 255, 255, 255));
            assert_eq!(image.pixels[1], Rgba(255, 255, 255, 255));
            assert_eq!(image.pixels[4], Rgba(255, 255, 255, 255));
        }
        other => panic!("unexpected output: {other:?}"),
    }
}

#[test]
fn neutral_invalid_cases_remain_blocking() {
    for (path, code) in [
        ("invalid/frame-in-11.pixl", "PIXL057"),
        ("invalid/empty-animation.pixl", "PIXL056"),
    ] {
        let text = corpus(path);
        let (output, diagnostics) = if path.contains("empty-animation") {
            process_animation(&text, path)
        } else {
            process_text(&text, path)
        };
        assert!(output.is_none(), "{path} unexpectedly rendered");
        assert!(
            diagnostics
                .0
                .iter()
                .any(|diagnostic| diagnostic.code == code),
            "{path}: {:?}",
            diagnostics.0
        );
    }
}

#[test]
fn all_neutral_valid_cases_are_accepted() {
    for path in ["valid/basic.pixl", "raster/layers.pixl", "svg/vector.pixl"] {
        let text = corpus(path);
        let (output, diagnostics) = process_text(&text, path);
        assert!(output.is_some(), "{path}: {:?}", diagnostics.0);
        assert!(!diagnostics.has_errors(), "{path}: {:?}", diagnostics.0);
    }
}

#[test]
fn public_export_helpers_write_expected_files() {
    let temp = std::env::temp_dir().join("pixl-motorru-export-tests");
    let _ = std::fs::remove_dir_all(&temp);
    std::fs::create_dir_all(&temp).unwrap();
    let (output, diagnostics) = process_text(&corpus("valid/basic.pixl"), "basic.pixl");
    assert!(!diagnostics.has_errors());
    let image = match output.unwrap() {
        Output::Raster(image) => image,
        other => panic!("unexpected output: {other:?}"),
    };
    let ppm = temp.join("image.ppm");
    write_raster_ppm(&image, &ppm).unwrap();
    assert!(std::fs::read(&ppm).unwrap().starts_with(b"P6\n2 1\n255\n"));

    let (output, diagnostics) = process_text(&corpus("svg/vector.pixl"), "vector.pixl");
    assert!(!diagnostics.has_errors());
    let svg = match output.unwrap() {
        Output::Svg(svg) => svg,
        other => panic!("unexpected output: {other:?}"),
    };
    let svg_path = temp.join("image.svg");
    write_svg(&svg, &svg_path).unwrap();
    assert!(std::fs::read_to_string(svg_path).unwrap().contains("<svg"));

    let (output, diagnostics) = process_animation(&corpus("animation/basic.pixl"), "basic.pixl");
    assert!(!diagnostics.has_errors());
    let animation = match output.unwrap() {
        Output::Animation(animation) => animation,
        other => panic!("unexpected output: {other:?}"),
    };
    let frames = temp.join("frames");
    write_animation_ppm(&animation, &frames).unwrap();
    assert!(frames.join("frame-0001-10ms.ppm").exists());
    let _ = std::fs::remove_dir_all(temp);
}

#[test]
fn malformed_syntax_is_blocking_and_keeps_source_location() {
    let text = "PIXL(1.1)\nCANVAS(1, 1)\nTIPE(bit)\nHEX8()\nLAYER(1)\n    pixel(1,, FFFFFFFF)\n";
    let (output, diagnostics) = process_text(text, "syntax.pixl");
    assert!(output.is_none());
    let diagnostic = diagnostics
        .0
        .iter()
        .find(|diagnostic| diagnostic.code == "PIXL001")
        .expect("syntax diagnostic");
    assert_eq!((diagnostic.line, diagnostic.character), (6, 5));

    let (output, diagnostics) = process_text(
        "PIXL(1.1)\nCANVAS(1, 1)\nTIPE(bit)\nHEX8()\nLAYER(1)\n    pixel(1, 1, FFFFFFFF\n",
        "unterminated.pixl",
    );
    assert!(output.is_none());
    assert!(
        diagnostics
            .0
            .iter()
            .any(|diagnostic| diagnostic.code == "PIXL001")
    );
}

#[test]
fn unsupported_color_system_and_invalid_components_are_blocking() {
    let text =
        "PIXL(1.1)\nCANVAS(1, 1)\nTIPE(bit)\nRGBA(3)\nLAYER(1)\npixel(1, 1, 255, nope, 0, 255)\n";
    let (output, diagnostics) = process_text(text, "color.pixl");
    assert!(output.is_none());
    assert!(
        diagnostics
            .0
            .iter()
            .any(|diagnostic| diagnostic.code == "PIXL014")
    );
    let text =
        "PIXL(1.1)\nCANVAS(1, 1)\nTIPE(bit)\nRGBA(2)\nLAYER(1)\npixel(1, 1, 255, nope, 0, 255)\n";
    let (output, diagnostics) = process_text(text, "component.pixl");
    assert!(output.is_none());
    assert!(
        diagnostics
            .0
            .iter()
            .any(|diagnostic| diagnostic.code == "PIXL035")
    );
}
