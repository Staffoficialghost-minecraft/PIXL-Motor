from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

from .document import SourceLocation


@dataclass(frozen=True)
class Canvas:
    width: int
    height: int


@dataclass(frozen=True)
class ColorRGBA8:
    red: int
    green: int
    blue: int
    alpha: int


class OutlinePosition(str, Enum):
    INTERIOR = "i"
    CENTER = "c"
    EXTERIOR = "e"


@dataclass(frozen=True)
class FillStyle:
    kind: str = "fill"


@dataclass(frozen=True)
class OutlineStyle:
    thickness: int = 1
    position: OutlinePosition = OutlinePosition.CENTER
    kind: str = "outline"


Style = FillStyle | OutlineStyle


@dataclass(frozen=True)
class Operation:
    name: str
    arguments: tuple[object, ...]
    location: SourceLocation


@dataclass(frozen=True)
class PixelOperation(Operation):
    x: int = 0
    y: int = 0
    color: ColorRGBA8 | None = None


@dataclass(frozen=True)
class InvisibleOperation(Operation):
    x: int = 0
    y: int = 0


@dataclass(frozen=True)
class RectangleOperation(Operation):
    x_range: tuple[int, int] = (0, 0)
    y_range: tuple[int, int] = (0, 0)
    color: ColorRGBA8 | None = None
    style: Style = FillStyle()


@dataclass(frozen=True)
class FillOperation(Operation):
    x_range: tuple[int, int] = (0, 0)
    y_range: tuple[int, int] = (0, 0)
    color: ColorRGBA8 | None = None
    write_policy: str = "UNDECLARED_ONLY"


@dataclass(frozen=True)
class ShapeOperation(Operation):
    geometry: tuple[object, ...] = ()
    color: ColorRGBA8 | None = None
    style: Style = FillStyle()


@dataclass(frozen=True)
class ProcessedLayer:
    number: int
    operations: tuple[Operation, ...]


@dataclass(frozen=True)
class ProcessedFrame:
    number: int
    duration_ms: int
    layers: tuple[ProcessedLayer, ...]


@dataclass(frozen=True)
class ProcessedDocument:
    language_version: str
    document_type: str
    color_system: str
    canvas: Canvas
    layers: tuple[ProcessedLayer, ...]
    frames: tuple[ProcessedFrame, ...] = ()


@dataclass(frozen=True)
class ProcessedAnimation:
    language_version: str
    document_type: str
    color_system: str
    canvas: Canvas
    frames: tuple[ProcessedFrame, ...]
