import sys
from pathlib import Path

import pixl_core


CORPUS = (
    Path(__file__).parents[3]
    / "PIXL-MotorPY"
    / "PIXL-MotorPY-v1-BETA"
    / "tests"
    / "corpus"
)


def test_core_import_does_not_load_graphical_modules():
    assert "PIL" not in sys.modules
    assert "tkinter" not in sys.modules


def test_process_text_returns_normalized_ir_without_images():
    text = (CORPUS / "valid" / "basic.pixl").read_text(encoding="utf-8")
    result, diagnostics = pixl_core.process_text(text, file="basic.pixl")

    assert not diagnostics.has_errors
    assert isinstance(result, pixl_core.ProcessedDocument)
    assert result.canvas == pixl_core.Canvas(2, 1)
    assert result.layers[0].operations[0].color == pixl_core.ColorRGBA8(255, 0, 0, 255)
    assert not hasattr(result, "pixels")


def test_fill_exposes_undeclared_only_policy():
    text = """PIXL(1.1)
CANVAS(2, 1)
TIPE(bit)
HEX8()
LAYER(1)
fill(1-2, 1-1, 00FF00FF)
"""
    result, diagnostics = pixl_core.process_text(text)

    assert not diagnostics.has_errors
    operation = result.layers[0].operations[0]
    assert isinstance(operation, pixl_core.FillOperation)
    assert operation.write_policy == "UNDECLARED_ONLY"


def test_animation_preserves_frames_layers_and_durations():
    text = (CORPUS / "animation" / "basic.pixl").read_text(encoding="utf-8")
    result, diagnostics = pixl_core.process_animation(text, file="basic.pixl")

    assert not diagnostics.has_errors
    assert isinstance(result, pixl_core.ProcessedAnimation)
    assert [(frame.number, frame.duration_ms) for frame in result.frames] == [(1, 10), (2, 20)]


def test_invalid_document_does_not_produce_processed_ir():
    result, diagnostics = pixl_core.process_text("PIXL(9.9)\n", file="bad.pixl")

    assert result is None
    assert diagnostics.has_errors


def test_groups_resolve_global_then_layer_with_late_declarations():
    text = """PIXL(1.1)
CANVAS(1, 1)
TIPE(bit)
HEX8()
GROUP = color(FF0000FF)
LAYER(1)
pixel(1, 1, color)
GROUP = color(00FF00FF)
"""
    result, diagnostics = pixl_core.process_text(text)

    assert not diagnostics.has_errors
    assert result.layers[0].operations[0].color == pixl_core.ColorRGBA8(0, 255, 0, 255)


def test_unknown_group_is_warning_and_transparent():
    text = """PIXL(1.1)
CANVAS(1, 1)
TIPE(bit)
HEX8()
LAYER(1)
pixel(1, 1, missing)
"""
    result, diagnostics = pixl_core.process_text(text)

    assert result is not None
    assert [item.code for item in diagnostics.items] == ["PIXL040"]
    assert result.layers[0].operations[0].color == pixl_core.ColorRGBA8(0, 0, 0, 0)


def test_rgba1_rgba2_and_hex8_normalize_to_same_color():
    documents = (
        ("HEX8()", "FF0080FF"),
        ("RGBA(2)", "255, 0, 128, 255"),
        ("RGBA(1)", "1, 0, 0.5019607843137255, 1"),
    )
    colors = []
    for color_system, literal in documents:
        text = f"""PIXL(1.1)
CANVAS(1, 1)
TIPE(bit)
{color_system}
LAYER(1)
pixel(1, 1, {literal})
"""
        result, diagnostics = pixl_core.process_text(text)
        assert not diagnostics.has_errors
        colors.append(result.layers[0].operations[0].color)

    assert colors == [pixl_core.ColorRGBA8(255, 0, 128, 255)] * 3


def test_shapes_styles_and_source_locations_are_preserved():
    text = """PIXL(1.1)
CANVAS(4, 4)
TIPE(vec)
HEX8()
LAYER(1)
rect(1-2, 1-2, FF0000FF, outline, 2, e)
circle(2, 2, 1, 00FF00FF, fill)
"""
    result, diagnostics = pixl_core.process_text(text, file="drawing.pixl")

    assert not diagnostics.has_errors
    rect, circle = result.layers[0].operations
    assert isinstance(rect, pixl_core.RectangleOperation)
    assert rect.style == pixl_core.OutlineStyle(2, pixl_core.OutlinePosition.EXTERIOR)
    assert isinstance(circle, pixl_core.ShapeOperation)
    assert circle.location == pixl_core.SourceLocation(7, 1)


def test_invalid_coordinates_keep_file_line_and_character():
    text = """PIXL(1.1)
CANVAS(1, 1)
TIPE(bit)
HEX8()
LAYER(1)
pixel(-1, 1, FFFFFFFF)
"""
    result, diagnostics = pixl_core.process_text(text, file="coordinates.pixl")

    assert result is None
    diagnostic = next(item for item in diagnostics.items if item.code == "PIXL030")
    assert (diagnostic.file, diagnostic.line, diagnostic.character) == ("coordinates.pixl", 6, 1)


def test_animation_preserves_group_resolution_and_operation_order():
    text = """PIXL(1.2)
CANVAS(1, 1)
TIPE(bit)
HEX8()
GROUP = base(0000FFFF)
FRAME(1, 10)
LAYER(1)
pixel(1, 1, base)
pixel(1, 1, FF0000FF)
"""
    result, diagnostics = pixl_core.process_animation(text)

    assert not diagnostics.has_errors
    operations = result.frames[0].layers[0].operations
    assert [operation.name for operation in operations] == ["pixel", "pixel"]
    assert operations[0].color == pixl_core.ColorRGBA8(0, 0, 255, 255)
    assert operations[1].color == pixl_core.ColorRGBA8(255, 0, 0, 255)
