from dataclasses import dataclass
import math

from .diagnostics import DiagnosticBag
from .document import ColorValue, Document, PixelState
from .resolver import Resolver


@dataclass(frozen=True)
class SceneCell:
    state: PixelState
    color: ColorValue | None


@dataclass(frozen=True)
class SceneLayer:
    number: int
    width: int
    height: int
    cells: tuple[SceneCell, ...]

    def cell(self, x: int, y: int) -> SceneCell:
        return self.cells[(y - 1) * self.width + (x - 1)]


@dataclass(frozen=True)
class RenderScene:
    width: int
    height: int
    layers: tuple[SceneLayer, ...]


def build_scene(document: Document, diagnostics: DiagnosticBag) -> RenderScene:
    if document.canvas is None:
        raise ValueError("cannot render a document without canvas")
    width, height = document.canvas
    resolver = Resolver()
    layers: list[SceneLayer] = []
    for layer in document.layers:
        states = [PixelState.UNDECLARED] * (width * height)
        colors: list[ColorValue | None] = [None] * (width * height)
        for command in layer.commands:
            if command.name == "inv":
                x, y = command.arguments
                _set_invisible(states, colors, (y - 1) * width + x - 1)
                continue
            color_arguments = command.arguments[2:]
            if command.name in {"circle", "ellipse", "polygon"}:
                mode = max(index for index, value in enumerate(command.arguments) if value in {"fill", "outline"})
                count = 1 if document.color_system == "HEX8" else 4
                color_arguments = command.arguments[mode - count:mode]
            color = _resolve_command_color(color_arguments, document, layer.groups, resolver, diagnostics, command.location)
            if command.name == "pixel":
                x, y = command.arguments[:2]
                _set_color(states, colors, (y - 1) * width + x - 1, color)
                continue
            if command.name in {"circle", "ellipse", "polygon"}:
                _draw_shape(command.arguments, command.name, color, states, colors, width, height, document.color_system)
                continue
            x_range, y_range = command.arguments[:2]
            style = command.arguments[3 if document.color_system == "HEX8" else 6:]
            if command.name == "rect" and style and style[0] == "outline":
                _draw_rect_outline(
                    x_range, y_range, style, color, states, colors, width, height
                )
                continue
            for y in range(y_range[0], y_range[1] + 1):
                for x in range(x_range[0], x_range[1] + 1):
                    index = (y - 1) * width + x - 1
                    selected = command.name == "rect"
                    if command.name == "fill":
                        selected = states[index] is PixelState.UNDECLARED
                    if selected:
                        _set_color(states, colors, index, color)
        layers.append(
            SceneLayer(
                layer.number,
                width,
                height,
                tuple(SceneCell(state, color) for state, color in zip(states, colors)),
            )
        )
    return RenderScene(width, height, tuple(layers))


def _draw_rect_outline(x_range, y_range, style, color, states, colors, width, height):
    thickness = style[1] if len(style) >= 3 else 1
    position = style[2] if len(style) >= 3 else "c"
    left, right = x_range[0] - 1, x_range[1]
    top, bottom = y_range[0] - 1, y_range[1]
    for y in range(1, height + 1):
        for x in range(1, width + 1):
            cx, cy = x - 0.5, y - 0.5
            inside = left <= cx <= right and top <= cy <= bottom
            distance = min(
                abs(cx - left), abs(cx - right), abs(cy - top), abs(cy - bottom)
            )
            if _outline_selected(inside, distance, thickness, position):
                _set_color(states, colors, (y - 1) * width + x - 1, color)


def _resolve_command_color(arguments, document, local_groups, resolver, diagnostics, location):
    if document.color_system == "HEX8":
        value = arguments[0]
    elif len(arguments) == 1 and isinstance(arguments[0], str):
        value = arguments[0]
    else:
        value = tuple(arguments)
    return resolver.resolve_color(value, document, local_groups, diagnostics, location)


def _shape_parts(arguments, color_system):
    mode = max(index for index, value in enumerate(arguments) if value in {"fill", "outline"})
    color_count = 1 if color_system == "HEX8" else 4
    return arguments[:mode - color_count], arguments[mode - color_count:mode], arguments[mode:]


def _draw_shape(arguments, name, color, states, colors, width, height, color_system):
    if color is None:
        return
    geometry, _, style = _shape_parts(arguments, color_system)
    mode = style[0]
    thickness = style[1] if len(style) >= 3 else 1
    position = style[2] if len(style) >= 3 else "c"
    for y in range(1, height + 1):
        for x in range(1, width + 1):
            cx, cy = x - 0.5, y - 0.5
            inside, distance = _shape_metrics(name, geometry, cx, cy)
            if mode == "fill":
                selected = inside
            else:
                selected = _outline_selected(inside, distance, thickness, position)
            if selected:
                _set_color(states, colors, (y - 1) * width + x - 1, color)


def _shape_metrics(name, geometry, cx, cy):
    if name == "circle":
        x, y, radius = geometry
        distance = abs(math.hypot(cx - x, cy - y) - radius)
        return math.hypot(cx - x, cy - y) <= radius, distance
    if name == "ellipse":
        x1, y1, x2, y2 = geometry
        center_x, center_y = (x1 + x2) / 2, (y1 + y2) / 2
        rx, ry = abs(x2 - x1) / 2, abs(y2 - y1) / 2
        value = ((cx - center_x) / rx) ** 2 + ((cy - center_y) / ry) ** 2
        distance = abs(math.sqrt(value) - 1) * min(rx, ry)
        return value <= 1, distance
    points = list(zip(geometry[::2], geometry[1::2]))
    inside = False
    for first, second in zip(points, points[1:] + points[:1]):
        if (first[1] > cy) != (second[1] > cy):
            crossing = (second[0] - first[0]) * (cy - first[1]) / (second[1] - first[1]) + first[0]
            if cx < crossing:
                inside = not inside
    distance = min(
        _distance_to_segment(cx, cy, first[0], first[1], second[0], second[1])
        for first, second in zip(points, points[1:] + points[:1])
    )
    return inside, distance


def _distance_to_segment(px, py, x1, y1, x2, y2):
    dx, dy = x2 - x1, y2 - y1
    if dx == dy == 0:
        return math.hypot(px - x1, py - y1)
    ratio = max(0, min(1, ((px - x1) * dx + (py - y1) * dy) / (dx * dx + dy * dy)))
    return math.hypot(px - (x1 + ratio * dx), py - (y1 + ratio * dy))


def _outline_selected(inside, distance, thickness, position):
    half = thickness / 2
    if position == "i":
        return inside and distance <= thickness
    if position == "e":
        return not inside and distance <= thickness
    return distance <= half


def _set_color(states, colors, index, color) -> None:
    states[index] = PixelState.DECLARED
    colors[index] = color


def _set_invisible(states, colors, index) -> None:
    states[index] = PixelState.INVISIBLE
    colors[index] = None
