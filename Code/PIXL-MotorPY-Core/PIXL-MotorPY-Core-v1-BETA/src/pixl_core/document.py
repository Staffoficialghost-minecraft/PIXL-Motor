from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum


class PixelState(str, Enum):
    UNDECLARED = "UNDECLARED"
    DECLARED = "DECLARED"
    INVISIBLE = "INVISIBLE"


@dataclass(frozen=True)
class SourceLocation:
    line: int
    character: int


@dataclass(frozen=True)
class ColorValue:
    red: int
    green: int
    blue: int
    alpha: int


@dataclass(frozen=True)
class ColorReference:
    value: object
    location: SourceLocation


@dataclass(frozen=True)
class Command:
    name: str
    arguments: tuple[object, ...]
    location: SourceLocation


@dataclass
class Layer:
    number: int
    location: SourceLocation
    groups: dict[str, ColorReference] = field(default_factory=dict)
    commands: list[Command] = field(default_factory=list)


@dataclass
class Frame:
    number: int
    duration_ms: int
    location: SourceLocation
    groups: dict[str, ColorReference] = field(default_factory=dict)
    layers: list[Layer] = field(default_factory=list)


@dataclass
class Document:
    version: str | None = None
    canvas: tuple[int, int] | None = None
    tipe: str | None = None
    color_system: str | None = None
    layers: list[Layer] = field(default_factory=list)
    global_groups: dict[str, ColorReference] = field(default_factory=dict)
    frames: list[Frame] = field(default_factory=list)


ParsedDocument = Document
