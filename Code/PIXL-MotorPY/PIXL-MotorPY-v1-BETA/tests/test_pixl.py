import sys
import builtins
import json
import pytest
from pathlib import Path
from xml.etree import ElementTree

sys.path.insert(0, str(Path(__file__).parents[1]))

from pixl.pipeline import process_text
from pixl.pipeline import process_text_svg
from pixl.pipeline import process_animation
from pixl.diagnostics import DiagnosticBag
from pixl.errors import ErrorDefinition, create_core_registry
from pixl.lexer import Lexer
from pixl.renderer import render_svg
from unittest.mock import patch
import main as pixl_cli


def test_fill_preserves_declared_and_invisible_pixels():
    image, diagnostics = process_text(
        """PIXL(1.1)
CANVAS(4, 4)
TIPE(bit)
HEX8()
LAYER(1)
pixel(1, 1, FF0000FF)
inv(2, 2)
fill(1-4, 1-4, 00FF00FF)
"""
    )
    assert not diagnostics.has_errors
    assert image is not None
    assert image.pixel(1, 1) == (255, 0, 0, 255)
    assert image.pixel(2, 2) == (0, 0, 0, 0)
    assert image.pixel(3, 3) == (0, 255, 0, 255)


def test_rect_is_inclusive_and_later_commands_overwrite():
    image, diagnostics = process_text(
        """PIXL(1.1)
CANVAS(3, 2)
TIPE(bit)
HEX8()
LAYER(1)
rect(1-2, 1-2, FF0000FF)
pixel(2, 1, 0000FFFF)
"""
    )
    assert not diagnostics.has_errors
    assert image is not None
    assert image.pixel(1, 1) == (255, 0, 0, 255)
    assert image.pixel(2, 1) == (0, 0, 255, 255)
    assert image.pixel(2, 2) == (255, 0, 0, 255)


def test_later_commands_overwrite_invisible_but_fill_does_not():
    image, diagnostics = process_text(
        """PIXL(1.1)
CANVAS(2, 1)
TIPE(bit)
HEX8()
LAYER(1)
inv(1, 1)
fill(1-2, 1-1, 00FF00FF)
pixel(1, 1, FF0000FF)
"""
    )
    assert not diagnostics.has_errors
    assert image is not None
    assert image.pixel(1, 1) == (255, 0, 0, 255)
    assert image.pixel(2, 1) == (0, 255, 0, 255)


def test_groups_resolve_with_layer_priority():
    image, diagnostics = process_text(
        """PIXL(1.1)
CANVAS(2, 1)
TIPE(bit)
HEX8()
GROUP = color(FF0000FF)
LAYER(1)
GROUP = color(00FF00FF)
pixel(1, 1, color)
pixel(2, 1, missing)
"""
    )
    assert image is not None
    assert image.pixel(1, 1) == (0, 255, 0, 255)
    assert image.pixel(2, 1) == (0, 0, 0, 0)
    assert any(item.code == "PIXL040" for item in diagnostics.items)


def test_invalid_document_does_not_render():
    image, diagnostics = process_text(
        """PIXL(9.9)
CANVAS(0, 2)
TIPE(bit)
HEX8()
LAYER(1)
pixel(1, 1, nope)
"""
    )
    assert image is None
    assert diagnostics.has_errors


def test_invalid_hex_literal_is_blocking():
    image, diagnostics = process_text(
        """PIXL(1.1)
CANVAS(1, 1)
TIPE(bit)
HEX8()
LAYER(1)
pixel(1, 1, GGGGGGGG)
"""
    )
    assert image is None
    assert any(item.code == "PIXL032" for item in diagnostics.items)


def test_polygon_geometry_rules_are_blocking():
    for filename, code in (
        ("degenerate.pixl", "PIXL039"),
        ("repeated-vertex.pixl", "PIXL038"),
    ):
        text = (Path(__file__).parent / "corpus" / "geometry" / filename).read_text(encoding="utf-8")
        image, diagnostics = process_text(text, file=filename)
        assert image is None
        assert any(item.code == code for item in diagnostics.items)


def test_corpus_manifest_cases():
    corpus = Path(__file__).parent / "corpus"
    manifest = json.loads((corpus / "manifest.json").read_text(encoding="utf-8"))
    for case in manifest["cases"]:
        text = (corpus / case["file"]).read_text(encoding="utf-8")
        if case["language"] == "1.2":
            result, diagnostics = process_animation(text, file=case["file"])
        else:
            result, diagnostics = process_text(text, file=case["file"])
        expected = set(case.get("expected_diagnostics", []))
        if expected:
            assert result is None
            assert expected <= {item.code for item in diagnostics.items}
        else:
            assert not diagnostics.has_errors
            assert result is not None


def test_animation_svg_is_rejected_explicitly():
    animation, diagnostics = process_animation(
        """PIXL(1.2)
CANVAS(1, 1)
TIPE(vec)
HEX8()
FRAME(1, 10)
LAYER(1)
pixel(1, 1, FFFFFFFF)
""",
        file="animation-svg.pixl",
    )
    assert animation is None
    assert any(item.code == "PIXL063" for item in diagnostics.items)


def test_cli_help_is_successful():
    with patch.object(pixl_cli.sys, "argv", ["main.py", "--help"]):
        assert pixl_cli.main() == 0


def test_layers_have_independent_states_and_compose_with_alpha():
    image, diagnostics = process_text(
        """PIXL(1.1)
CANVAS(1, 1)
TIPE(bit)
HEX8()
LAYER(1)
inv(1, 1)
LAYER(2)
pixel(1, 1, FF000080)
"""
    )
    assert not diagnostics.has_errors
    assert image is not None
    assert image.pixel(1, 1) == (255, 0, 0, 128)


def test_groups_can_be_declared_after_use_and_unknown_group_is_warning():
    image, diagnostics = process_text(
        """PIXL(1.1)
CANVAS(2, 1)
TIPE(bit)
HEX8()
LAYER(1)
pixel(1, 1, late)
pixel(2, 1, missing)
GROUP = late(010203FF)
"""
    )
    assert image is not None
    assert image.pixel(1, 1) == (1, 2, 3, 255)
    assert image.pixel(2, 1) == (0, 0, 0, 0)
    assert [item.code for item in diagnostics.items] == ["PIXL040"]


def test_hex_shaped_lowercase_group_name_is_an_identifier_in_group_declaration():
    image, diagnostics = process_text(
        """PIXL(1.1)
CANVAS(1, 1)
TIPE(bit)
HEX8()
GROUP = deadbeef(010203FF)
LAYER(1)
pixel(1, 1, deadbeef)
"""
    )
    assert not diagnostics.has_errors
    assert image is not None
    assert image.pixel(1, 1) == (1, 2, 3, 255)


def test_color_systems_produce_equivalent_rgba():
    documents = [
        ("HEX8()", "FF000080"),
        ("RGBA(2)", "255, 0, 0, 128"),
        ("RGBA(1)", "1, 0, 0, 0.5019607843137255"),
    ]
    pixels = []
    for system, color in documents:
        image, diagnostics = process_text(
            f"""PIXL(1.1)
CANVAS(1, 1)
TIPE(bit)
{system}
LAYER(1)
pixel(1, 1, {color})
"""
        )
        assert not diagnostics.has_errors
        assert image is not None
        pixels.append(image.pixel(1, 1))
    assert pixels == [(255, 0, 0, 128)] * 3


def test_malformed_arguments_and_duplicate_declarations_are_blocking():
    image, diagnostics = process_text(
        """PIXL(1.1)
PIXL(1.1)
CANVAS(2 2)
CANVAS(2, 2)
TIPE(bit)
HEX8()
LAYER(1)
pixel(-1, 1, FF0000FF)
"""
    )
    assert image is None
    assert diagnostics.has_errors
    assert {"PIXL001", "PIXL021", "PIXL030"} <= {item.code for item in diagnostics.items}


def test_duplicate_color_system_and_layer_are_blocking():
    image, diagnostics = process_text(
        """PIXL(1.1)
CANVAS(1, 1)
TIPE(bit)
RGBA(2)
HEX8()
LAYER(1)
LAYER(1)
"""
    )
    assert image is None
    assert diagnostics.has_errors
    assert {"PIXL017", "PIXL020"} <= {item.code for item in diagnostics.items}


def test_invalid_group_literal_is_error_not_transparency_warning():
    image, diagnostics = process_text(
        """PIXL(1.1)
CANVAS(1, 1)
TIPE(bit)
HEX8()
GROUP = bad(GGGGGGGG)
LAYER(1)
pixel(1, 1, bad)
"""
    )
    assert image is None
    assert any(item.code == "PIXL033" for item in diagnostics.items)
    assert not any(item.code == "PIXL040" for item in diagnostics.items)


def test_diagnostics_keep_file_line_and_character():
    image, diagnostics = process_text(
        """PIXL(1.1)
CANVAS(1, 1)
TIPE(bit)
HEX8()
LAYER(1)
pixel(2, 1, FF0000FF)
""",
        file="drawing.pixl",
    )
    assert image is None
    diagnostic = next(item for item in diagnostics.items if item.code == "PIXL030")
    assert (diagnostic.file, diagnostic.line, diagnostic.character) == ("drawing.pixl", 6, 1)


def test_invalid_documents_never_call_renderer():
    with patch("pixl.pipeline.render") as render_mock:
        image, diagnostics = process_text("PIXL(9.9)\n", file="bad.pixl")
    assert image is None
    assert diagnostics.has_errors
    render_mock.assert_not_called()


def test_lexer_preserves_comments_lines_and_token_locations():
    diagnostics = DiagnosticBag("comments.pixl")
    tokens = Lexer(diagnostics).tokenize(
        "# comment\n\tPIXL(1.1) # trailing\n\nCANVAS(2, 2)\n"
    )
    meaningful = [token for token in tokens if token.kind not in {"NEWLINE", "EOF"}]
    assert [(token.text, token.line, token.character) for token in meaningful[:4]] == [
        ("PIXL", 2, 2),
        ("(", 2, 6),
        ("1.1", 2, 7),
        (")", 2, 10),
    ]
    assert [(token.text, token.line) for token in meaningful[-6:]] == [
        ("CANVAS", 4),
        ("(", 4),
        ("2", 4),
        (",", 4),
        ("2", 4),
        (")", 4),
    ]
    assert not diagnostics.items


def test_lexer_reports_unexpected_characters_without_raising():
    diagnostics = DiagnosticBag("invalid.pixl")
    tokens = Lexer(diagnostics).tokenize("PIXL(1.1) @\n")
    assert any(token.kind == "INVALID" and token.text == "@" for token in tokens)
    assert [(item.code, item.file, item.line, item.character) for item in diagnostics.items] == [
        ("PIXL002", "invalid.pixl", 1, 11)
    ]


def test_negative_numbers_and_decimals_are_tokenized():
    tokens = Lexer().tokenize("pixel(-1, -2.5, FF0000FF)\n")
    values = [token.text for token in tokens if token.kind in {"NUMBER", "-", "HEX"}]
    assert values == ["-", "1", "-", "2.5", "FF0000FF"]


def test_parser_recovers_after_malformed_instruction():
    image, diagnostics = process_text(
        """PIXL(1.1)
CANVAS(3, 1)
TIPE(bit)
HEX8()
LAYER(1)
pixel(1, 1, FF0000FF)
pixel(2 2 FF0000FF)
pixel(3, 1, 00FF00FF)
"""
    )
    assert image is None
    assert any(item.code == "PIXL001" and item.line == 7 for item in diagnostics.items)
    assert any(item.line == 8 for item in diagnostics.items) is False


def test_parser_reports_unclosed_unexpected_and_comma_errors():
    image, diagnostics = process_text(
        """PIXL(1.1)
CANVAS(1, 1)
TIPE(bit)
HEX8()
LAYER(1)
pixel(1, 1, FF0000FF
pixel(1,, 1, FF0000FF)
pixel(1, 1, FF0000FF,))
"""
    )
    assert image is None
    codes = [item.code for item in diagnostics.items]
    assert codes.count("PIXL001") >= 3


def test_comments_do_not_change_rendered_result():
    plain, plain_diagnostics = process_text(
        """PIXL(1.1)
CANVAS(1, 1)
TIPE(bit)
HEX8()
LAYER(1)
pixel(1, 1, FF0000FF)
"""
    )
    commented, commented_diagnostics = process_text(
        """# header
PIXL(1.1) # version

CANVAS(1, 1)
TIPE(bit)
HEX8() # colors
LAYER(1)
# pixel
pixel(1, 1, FF0000FF) # red
"""
    )
    assert not plain_diagnostics.has_errors
    assert not commented_diagnostics.has_errors
    assert plain is not None and commented is not None
    assert plain.pixels == commented.pixels


def test_cli_reports_missing_file_without_python_exception(monkeypatch, capsys):
    monkeypatch.setattr("sys.argv", ["main.py", "does-not-exist.pixl"])
    assert pixl_cli.main() == 1
    assert "No se encontró el archivo PIXL" in capsys.readouterr().out


def test_process_file_reports_read_errors_and_processes_valid_file(tmp_path):
    from pixl.pipeline import process_file

    missing, missing_diagnostics = process_file(tmp_path / "missing.pixl")
    assert missing is None
    assert any(item.code == "PIXL060" for item in missing_diagnostics.items)

    source = tmp_path / "image.pixl"
    source.write_text(
        "PIXL(1.1)\nCANVAS(1,1)\nTIPE(bit)\nHEX8()\n"
        "LAYER(1)\npixel(1,1,FFFFFFFF)\n",
        encoding="utf-8",
    )
    image, diagnostics = process_file(source)
    assert not diagnostics.has_errors
    assert image is not None


def test_core_diagnostic_registry_is_extensible_without_code_collisions():
    registry = create_core_registry()
    assert len(registry.definitions()) >= 10
    assert registry.get("418").description == "El servidor es un árbol."
    registry.register(ErrorDefinition("RENDER001", "render", "Error de renderizado"))
    assert registry.get("RENDER001") is not None
    diagnostics = DiagnosticBag(registry=registry)
    diagnostics.error("418", "árbol")
    assert diagnostics.items[0].category == "internal"
    assert diagnostics.items[0].details == "Diagnóstico humorístico reservado."
    with pytest.raises(ValueError):
        registry.register(ErrorDefinition("418", "render", "colisión"))


def test_gui_module_is_importable_without_starting_a_window():
    from pixl.gui import MotorPyGui, launch_gui

    assert callable(launch_gui)
    assert MotorPyGui.__name__ == "MotorPyGui"


def test_example_document_renders_non_empty_pixels():
    image, diagnostics = process_text(
        """PIXL(1.1)
CANVAS(16, 16)
TIPE(bit)
HEX8()

LAYER(1)
    pixel(1, 1, FF0000FF)
    pixel(2, 1, 00FF00FF)
    pixel(3, 1, 0000FFFF)
    rect(5-12, 5-12, FFFFFFFF)
LAYER(2)
    fill(1-16, 1-16, F0F0F0FF)
"""
    )
    assert not diagnostics.has_errors
    assert image is not None
    assert image.width == 16 and image.height == 16
    assert all(alpha > 0 for _, _, _, alpha in image.pixels)


def test_circle_and_rect_outline_render_with_center_rule():
    image, diagnostics = process_text(
        """PIXL(1.1)
CANVAS(5, 5)
TIPE(bit)
HEX8()
LAYER(1)
circle(3, 3, 1, FF0000FF, fill)
rect(1-5, 1-5, 00FF00FF, outline, 1, c)
"""
    )
    assert not diagnostics.has_errors
    assert image is not None
    assert image.pixel(3, 3) == (255, 0, 0, 255)
    assert image.pixel(1, 1) == (0, 255, 0, 255)


def test_rect_outline_positions_affect_interior_and_exterior_bands():
    images = {}
    for position in ("i", "c", "e"):
        image, diagnostics = process_text(
            f"""PIXL(1.1)
CANVAS(9, 9)
TIPE(bit)
HEX8()
LAYER(1)
rect(3-7, 3-7, FFFFFFFF, outline, 3, {position})
"""
        )
        assert not diagnostics.has_errors
        assert image is not None
        images[position] = image

    assert images["i"].pixel(2, 2) == (0, 0, 0, 0)
    assert images["e"].pixel(2, 2) == (255, 255, 255, 255)
    assert images["i"].pixel(5, 5) == (255, 255, 255, 255)
    assert images["e"].pixel(5, 5) == (0, 0, 0, 0)


def test_independent_animation_frames_and_durations():
    from pixl.pipeline import process_animation

    frames, diagnostics = process_animation(
        """PIXL(1.2)
CANVAS(2, 1)
TIPE(bit)
HEX8()
FRAME(1, 10)
LAYER(1)
pixel(1, 1, FF0000FF)
FRAME(2, 20)
LAYER(1)
pixel(2, 1, 0000FFFF)
"""
    )
    assert not diagnostics.has_errors
    assert frames is not None
    assert [(frame.number, frame.duration_ms) for frame in frames] == [(1, 10), (2, 20)]
    assert frames[0].image.pixel(1, 1) == (255, 0, 0, 255)
    assert frames[1].image.pixel(1, 1) == (0, 0, 0, 0)


def test_indented_animation_syntax_is_accepted_with_duration():
    from pixl.pipeline import process_animation

    frames, diagnostics = process_animation(
        """PIXL(1.2)
CANVAS(2, 1)
TIPE(bit)
HEX8()

FRAME(1, 10)
    LAYER(1)
        pixel(1, 1, FF0000FF)
FRAME(2, 10)
    LAYER(1)
        pixel(2, 1, 0000FFFF)
"""
    )
    assert not diagnostics.has_errors
    assert frames is not None
    assert [frame.number for frame in frames] == [1, 2]


def test_animation_requires_frames_and_validates_frame_layers():
    from pixl.pipeline import process_animation

    for source, codes in [
        ("PIXL(1.2)\nCANVAS(1,1)\nTIPE(bit)\nHEX8()\n", {"PIXL056"}),
        ("PIXL(1.2)\nCANVAS(1,1)\nTIPE(bit)\nHEX8()\nFRAME(1,1)\nLAYER(0)\npixel(1,1,FFFFFFFF)\nLAYER(0)\npixel(1,1,FFFFFFFF)\n", {"PIXL016", "PIXL017"}),
    ]:
        result, diagnostics = process_animation(source)
        assert result is None
        assert codes <= {item.code for item in diagnostics.items}


def test_version_specific_frame_rules_and_svg_rejection():
    from pixl.pipeline import process_animation, process_text_svg

    source = "PIXL(1.1)\nCANVAS(1,1)\nTIPE(bit)\nHEX8()\nFRAME(1,1)\nLAYER(1)\npixel(1,1,FFFFFFFF)\n"
    result, diagnostics = process_text(source)
    assert result is None
    assert any(item.code == "PIXL057" for item in diagnostics.items)
    animation, animation_diagnostics = process_animation(source)
    assert animation is None
    assert any(item.code == "PIXL058" for item in animation_diagnostics.items)

    svg, svg_diagnostics = process_text_svg(source.replace("1.1", "1.2"))
    assert svg is None
    assert any(item.code == "PIXL054" for item in svg_diagnostics.items)


def test_tipe_vec_renders_svg_automatically():
    svg, diagnostics = process_text(
        """PIXL(1.1)
CANVAS(2, 1)
TIPE(vec)
HEX8()
LAYER(1)
    pixel(1, 1, FF0000FF)
    pixel(2, 1, 0000FFFF)
"""
    )
    assert not diagnostics.has_errors
    assert isinstance(svg, str)
    assert "<svg" in svg
    assert 'width="2" height="1"' in svg


def test_tipe_vec_svg_contains_visible_rectangles_for_gui_preview():
    svg, diagnostics = process_text(
        """PIXL(1.1)
CANVAS(16, 16)
TIPE(vec)
HEX8()
LAYER(1)
pixel(1, 1, FF0000FF)
rect(5-12, 5-12, FFFFFFFF)
"""
    )
    assert not diagnostics.has_errors
    assert isinstance(svg, str)
    assert '<rect x="0" y="0"' in svg
    assert 'fill="#FFFFFF"' in svg


def test_svg_preserves_canvas_layers_rectangles_and_transparency():
    text = """PIXL(1.1)
CANVAS(3, 2)
TIPE(bit)
HEX8()
LAYER(1)
rect(1-2, 1-1, FF000080)
inv(3, 1)
LAYER(2)
pixel(1, 2, 00FF00FF)
"""
    image, diagnostics = process_text(text)
    assert not diagnostics.has_errors
    assert image is not None
    svg = _svg_for(text)
    ElementTree.fromstring(svg)
    assert 'width="3" height="2" viewBox="0 0 3 2"' in svg
    assert svg.count("<g data-layer=") == 2
    assert '<rect x="0" y="0" width="2" height="1" fill="#FF0000" fill-opacity="0.501961"/>' in svg
    assert '<rect x="0" y="1" width="1" height="1" fill="#00FF00"/>' in svg
    assert 'x="2" y="0"' not in svg
    assert svg.index('data-layer="1"') < svg.index('data-layer="2"')


def test_svg_backend_matches_declared_raster_pixels_and_does_not_import_pillow():
    text = """PIXL(1.1)
CANVAS(2, 1)
TIPE(bit)
HEX8()
LAYER(1)
pixel(1, 1, FF0000FF)
pixel(2, 1, 00000080)
"""
    image, diagnostics = process_text(text)
    assert not diagnostics.has_errors
    assert image is not None
    svg = _svg_for(text)
    assert image.pixel(1, 1) == (255, 0, 0, 255)
    assert image.pixel(2, 1) == (0, 0, 0, 128)
    assert 'fill="#FF0000"' in svg
    assert 'fill="#000000" fill-opacity="0.501961"' in svg
    assert "Pillow" not in svg


def test_svg_pipeline_does_not_import_pillow():
    text = """PIXL(1.1)
CANVAS(1, 1)
TIPE(bit)
HEX8()
LAYER(1)
pixel(1, 1, FF0000FF)
"""
    original_import = builtins.__import__

    def reject_pillow(name, *args, **kwargs):
        if name == "PIL" or name.startswith("PIL."):
            raise AssertionError("SVG backend must not import Pillow")
        return original_import(name, *args, **kwargs)

    with patch("builtins.__import__", side_effect=reject_pillow):
        svg, diagnostics = process_text_svg(text)
    assert not diagnostics.has_errors
    assert svg is not None


def test_invalid_document_cannot_be_rendered_as_svg():
    text = "PIXL(9.9)\n"
    svg, diagnostics = process_text_svg(text)
    assert svg is None
    assert diagnostics.has_errors


def _svg_for(text: str) -> str:
    from pixl.diagnostics import DiagnosticBag
    from pixl.lexer import Lexer
    from pixl.parser import Parser
    from pixl.validator import Validator

    diagnostics = DiagnosticBag("test.pixl")
    document = Parser(Lexer(diagnostics).tokenize(text), diagnostics).parse()
    Validator().validate(document, diagnostics)
    assert not diagnostics.has_errors
    return render_svg(document, diagnostics)
