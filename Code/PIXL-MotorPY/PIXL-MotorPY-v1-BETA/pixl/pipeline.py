from .diagnostics import DiagnosticBag
from .document import Command, Document, Layer, SourceLocation
from .renderer import RenderedImage, render, render_svg
from pathlib import Path

from .document import Animation, AnimationFrame
from pixl_core import (
    FillOperation,
    InvisibleOperation,
    OutlineStyle,
    PixelOperation,
    ProcessedAnimation,
    ProcessedDocument,
    RectangleOperation,
    ShapeOperation,
    process_animation as core_process_animation,
    process_animation_file as core_process_animation_file,
    process_file as core_process_file,
    process_text as core_process_text,
)


def process_text(text: str, *, file: str = "<string>") -> tuple[RenderedImage | str | None, DiagnosticBag]:
    processed, diagnostics = core_process_text(text, file=file)
    if diagnostics.has_errors:
        return None, diagnostics
    legacy = _legacy_document(processed)
    if legacy.tipe == "vec":
        return render_svg(legacy, diagnostics), diagnostics
    return render(legacy, diagnostics), diagnostics


def process_text_svg(text: str, *, file: str = "<string>") -> tuple[str | None, DiagnosticBag]:
    processed, diagnostics = core_process_text(text, file=file)
    if diagnostics.has_errors:
        return None, diagnostics
    return render_svg(_legacy_document(processed), diagnostics), diagnostics


def process_animation(text: str, *, file: str = "<string>") -> tuple[Animation | None, DiagnosticBag]:
    processed, diagnostics = core_process_animation(text, file=file)
    if diagnostics.has_errors:
        return None, diagnostics
    results: list[AnimationFrame] = []
    for frame in processed.frames:
        frame_document = _legacy_frame_document(processed, frame)
        results.append(AnimationFrame(frame.number, frame.duration_ms, render(frame_document, diagnostics)))
    return Animation(tuple(results)), diagnostics


def process_file(path: str | Path, *, file: str | None = None) -> tuple[RenderedImage | str | None, DiagnosticBag]:
    processed, diagnostics = core_process_file(path, file=file)
    if processed is None:
        return None, diagnostics
    legacy = _legacy_document(processed)
    return (
        render_svg(legacy, diagnostics)
        if legacy.tipe == "vec"
        else render(legacy, diagnostics),
        diagnostics,
    )


def process_animation_file(path: str | Path, *, file: str | None = None) -> tuple[Animation | None, DiagnosticBag]:
    processed, diagnostics = core_process_animation_file(path, file=file)
    if processed is None:
        return None, diagnostics
    results = [
        AnimationFrame(
            frame.number,
            frame.duration_ms,
            render(_legacy_frame_document(processed, frame), diagnostics),
        )
        for frame in processed.frames
    ]
    return Animation(tuple(results)), diagnostics


def _legacy_document(processed: ProcessedDocument) -> Document:
    return Document(
        version=processed.language_version,
        canvas=(processed.canvas.width, processed.canvas.height),
        tipe=processed.document_type,
        color_system="HEX8",
        layers=[_legacy_layer(layer) for layer in processed.layers],
    )


def _legacy_frame_document(processed: ProcessedAnimation, frame) -> Document:
    return Document(
        version="1.1",
        canvas=(processed.canvas.width, processed.canvas.height),
        tipe=processed.document_type,
        color_system="HEX8",
        layers=[_legacy_layer(layer) for layer in frame.layers],
    )


def _legacy_layer(processed_layer) -> Layer:
    return Layer(
        number=processed_layer.number,
        location=SourceLocation(1, 1),
        commands=[_legacy_command(operation) for operation in processed_layer.operations],
    )


def _legacy_command(operation) -> Command:
    if isinstance(operation, InvisibleOperation):
        return Command(
            "inv",
            (operation.x, operation.y),
            operation.location,
        )
    color = operation.color
    literal = (
        f"{color.red:02X}{color.green:02X}{color.blue:02X}{color.alpha:02X}"
        if color is not None
        else "00000000"
    )
    if isinstance(operation, PixelOperation):
        arguments = (operation.x, operation.y, literal)
    elif isinstance(operation, FillOperation):
        arguments = (operation.x_range, operation.y_range, literal)
    elif isinstance(operation, RectangleOperation):
        arguments = (operation.x_range, operation.y_range, literal, *_style_arguments(operation))
    elif isinstance(operation, ShapeOperation):
        arguments = (*operation.geometry, literal, *_style_arguments(operation))
    else:
        raise TypeError(f"unsupported normalized operation: {type(operation).__name__}")
    return Command(operation.name, tuple(arguments), operation.location)


def _style_arguments(operation) -> tuple[object, ...]:
    if isinstance(operation.style, OutlineStyle):
        return ("outline", operation.style.thickness, operation.style.position.value)
    return ("fill",)
