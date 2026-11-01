from __future__ import annotations

from .diagnostics import DiagnosticBag
from .document import Command, Document, Layer, SourceLocation
from .ir import (
    Canvas,
    ColorRGBA8,
    FillOperation,
    FillStyle,
    InvisibleOperation,
    Operation,
    OutlinePosition,
    OutlineStyle,
    PixelOperation,
    ProcessedAnimation,
    ProcessedDocument,
    ProcessedFrame,
    ProcessedLayer,
    RectangleOperation,
    ShapeOperation,
)
from .resolver import Resolver


def normalize_document(
    document: Document, diagnostics: DiagnosticBag
) -> ProcessedDocument:
    _require_document_metadata(document)
    return ProcessedDocument(
        language_version=document.version or "",
        document_type=document.tipe or "",
        color_system=document.color_system or "",
        canvas=Canvas(*document.canvas),
        layers=tuple(
            _normalize_layer(layer, document, {}, diagnostics)
            for layer in document.layers
        ),
    )


def normalize_animation(
    document: Document, diagnostics: DiagnosticBag
) -> ProcessedAnimation:
    _require_document_metadata(document)
    frames = tuple(
        ProcessedFrame(
            number=frame.number,
            duration_ms=frame.duration_ms,
            layers=tuple(
                _normalize_layer(layer, document, frame.groups, diagnostics)
                for layer in frame.layers
            ),
        )
        for frame in document.frames
    )
    return ProcessedAnimation(
        language_version=document.version or "",
        document_type=document.tipe or "",
        color_system=document.color_system or "",
        canvas=Canvas(*document.canvas),
        frames=frames,
    )


def _require_document_metadata(document: Document) -> None:
    if document.canvas is None or document.version is None:
        raise ValueError("validated document metadata is incomplete")


def _normalize_layer(
    layer: Layer,
    document: Document,
    frame_groups,
    diagnostics: DiagnosticBag,
) -> ProcessedLayer:
    resolver = Resolver()
    operations: list[Operation] = []
    for command in layer.commands:
        operations.append(
            _normalize_command(
                command,
                document,
                layer.groups,
                frame_groups,
                resolver,
                diagnostics,
            )
        )
    return ProcessedLayer(layer.number, tuple(operations))


def _normalize_command(
    command: Command,
    document: Document,
    layer_groups,
    frame_groups,
    resolver: Resolver,
    diagnostics: DiagnosticBag,
) -> Operation:
    args = command.arguments
    if command.name == "inv":
        return InvisibleOperation(
            name="inv",
            arguments=args,
            location=command.location,
            x=args[0],
            y=args[1],
        )
    if command.name == "pixel":
        return PixelOperation(
            name="pixel",
            arguments=args,
            location=command.location,
            x=args[0],
            y=args[1],
            color=_color(args[2:], document, layer_groups, frame_groups, resolver, diagnostics, command.location),
        )
    if command.name in {"rect", "fill"}:
        color_end = 3 if document.color_system == "HEX8" else 6
        color = _color(args[2:color_end], document, layer_groups, frame_groups, resolver, diagnostics, command.location)
        if command.name == "fill":
            return FillOperation(
                name="fill",
                arguments=args,
                location=command.location,
                x_range=args[0],
                y_range=args[1],
                color=color,
            )
        style = _style(args[color_end:])
        return RectangleOperation(
            name="rect",
            arguments=args,
            location=command.location,
            x_range=args[0],
            y_range=args[1],
            color=color,
            style=style,
        )
    mode = max(index for index, value in enumerate(args) if value in {"fill", "outline"})
    color_count = 1 if document.color_system == "HEX8" else 4
    color_start = mode - color_count
    color = _color(args[color_start:mode], document, layer_groups, frame_groups, resolver, diagnostics, command.location)
    return ShapeOperation(
        name=command.name,
        arguments=args,
        location=command.location,
        geometry=tuple(args[:color_start]),
        color=color,
        style=_style(args[mode:]),
    )


def _color(arguments, document, layer_groups, frame_groups, resolver, diagnostics, location):
    if document.color_system == "HEX8":
        value = arguments[0]
    elif len(arguments) == 1 and isinstance(arguments[0], str):
        value = arguments[0]
    else:
        value = tuple(arguments)
    resolved = resolver.resolve_color(
        value,
        Document(
            version=document.version,
            canvas=document.canvas,
            tipe=document.tipe,
            color_system=document.color_system,
            global_groups={
                **document.global_groups,
                **frame_groups,
            },
        ),
        layer_groups,
        diagnostics,
        location,
    )
    if resolved is None:
        return None
    return ColorRGBA8(resolved.red, resolved.green, resolved.blue, resolved.alpha)


def _style(values) -> FillStyle | OutlineStyle:
    if not values or values[0] == "fill":
        return FillStyle()
    return OutlineStyle(
        thickness=values[1] if len(values) >= 2 else 1,
        position=OutlinePosition(values[2]) if len(values) >= 3 else OutlinePosition.CENTER,
    )
