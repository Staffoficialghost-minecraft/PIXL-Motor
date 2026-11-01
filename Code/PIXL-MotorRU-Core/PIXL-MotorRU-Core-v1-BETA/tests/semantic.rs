use pixl_motorru_core::{Operation, ProcessedDocument, process_animation, process_text};

const CORPUS: &str = "../../PIXL-MotorPY/PIXL-MotorPY-v1-BETA/tests/corpus/";

fn corpus(path: &str) -> String {
    std::fs::read_to_string(format!("{CORPUS}{path}")).expect("neutral corpus case")
}

#[test]
fn neutral_valid_documents_are_accepted_without_rendering() {
    for path in ["valid/basic.pixl", "raster/layers.pixl", "svg/vector.pixl"] {
        let (document, diagnostics) = process_text(&corpus(path), path);
        assert!(document.is_some(), "{path}: {:?}", diagnostics.0);
        assert!(!diagnostics.has_errors(), "{path}: {:?}", diagnostics.0);
    }
}

#[test]
fn neutral_invalid_documents_are_rejected() {
    for path in [
        "invalid/frame-in-11.pixl",
        "geometry/degenerate.pixl",
        "geometry/repeated-vertex.pixl",
    ] {
        let (document, diagnostics) = process_text(&corpus(path), path);
        assert!(document.is_none(), "{path} unexpectedly accepted");
        assert!(diagnostics.has_errors(), "{path}: {:?}", diagnostics.0);
    }
}

#[test]
fn colors_groups_and_operation_order_match_contract() {
    let text = "PIXL(1.2)\nCANVAS(1,1)\nTIPE(bit)\nHEX8()\nGROUP = color(FF0000FF)\nFRAME(1,10)\nGROUP = color(00FF00FF)\nLAYER(1)\nGROUP = color(0000FFFF)\npixel(1,1,color)\npixel(1,1,FFFFFFFF)\n";
    let (animation, diagnostics) = process_animation(text, "groups.pixl");
    assert!(!diagnostics.has_errors(), "{:?}", diagnostics.0);
    let operations = &animation.unwrap().frames[0].layers[0].operations;
    assert!(matches!(
        operations[0],
        Operation::Pixel {
            color: pixl_motorru_core::ColorRGBA8 {
                red: 0,
                green: 0,
                blue: 255,
                alpha: 255
            },
            ..
        }
    ));
    assert!(matches!(
        operations[1],
        Operation::Pixel {
            color: pixl_motorru_core::ColorRGBA8 {
                red: 255,
                green: 255,
                blue: 255,
                alpha: 255
            },
            ..
        }
    ));
}

#[test]
fn invalid_coordinates_preserve_source_location() {
    let text = "PIXL(1.1)\nCANVAS(1,1)\nTIPE(bit)\nHEX8()\nLAYER(1)\npixel(-1,1,FFFFFFFF)\n";
    let (document, diagnostics) = process_text(text, "coordinates.pixl");
    assert!(document.is_none());
    let diagnostic = diagnostics
        .0
        .iter()
        .find(|item| item.code == "PIXL030")
        .unwrap();
    assert_eq!(
        (
            diagnostic.file.as_str(),
            diagnostic.line,
            diagnostic.character
        ),
        ("coordinates.pixl", 6, 1)
    );
}

#[test]
fn normalized_document_is_renderer_free() {
    let (document, diagnostics) = process_text(&corpus("valid/basic.pixl"), "basic.pixl");
    assert!(!diagnostics.has_errors());
    let document: ProcessedDocument = document.unwrap();
    assert_eq!((document.canvas.width, document.canvas.height), (2, 1));
    assert!(matches!(
        document.layers[0].operations[0],
        Operation::Pixel { .. }
    ));
}

#[test]
fn rgba_polygon_and_outline_are_normalized() {
    let text = "PIXL(1.1)\nCANVAS(4,4)\nTIPE(bit)\nRGBA(2)\nLAYER(1)\npolygon(1,1,4,1,1,4,255,0,0,128,outline,1,e)\n";
    let (document, diagnostics) = process_text(text, "polygon.pixl");
    assert!(!diagnostics.has_errors(), "{:?}", diagnostics.0);
    let operation = &document.unwrap().layers[0].operations[0];
    assert!(matches!(
        operation,
        Operation::Polygon {
            color: pixl_motorru_core::ColorRGBA8 {
                red: 255,
                green: 0,
                blue: 0,
                alpha: 128
            },
            style: pixl_motorru_core::Style::Outline {
                thickness: 1,
                position: pixl_motorru_core::OutlinePosition::Exterior
            },
            ..
        }
    ));
}
