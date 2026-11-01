from dataclasses import dataclass

from .diagnostics import DiagnosticBag
from .document import Document
from .render_model import build_scene


@dataclass(frozen=True)
class RenderedImage:
    width: int
    height: int
    pixels: tuple[tuple[int, int, int, int], ...]

    def pixel(self, x: int, y: int) -> tuple[int, int, int, int]:
        return self.pixels[(y - 1) * self.width + (x - 1)]


def render(document: Document, diagnostics: DiagnosticBag) -> RenderedImage:
    scene = build_scene(document, diagnostics)
    width, height = scene.width, scene.height
    final = [(0, 0, 0, 0)] * (width * height)
    for layer in scene.layers:
        final = [
            _over(background, layer.cells[index].color)
            for index, background in enumerate(final)
        ]
    return RenderedImage(width, height, tuple(final))


def render_svg(document: Document, diagnostics: DiagnosticBag) -> str:
    from .renderers.svg_backend import SvgBackend

    return SvgBackend().to_svg(build_scene(document, diagnostics))


def _over(background: tuple[int, int, int, int], foreground) -> tuple[int, int, int, int]:
    if foreground is None:
        return background
    source_alpha = foreground.alpha / 255
    destination_alpha = background[3] / 255
    output_alpha = source_alpha + destination_alpha * (1 - source_alpha)
    if output_alpha == 0:
        return (0, 0, 0, 0)
    return tuple(
        round((source * source_alpha + destination * destination_alpha * (1 - source_alpha)) / output_alpha)
        for source, destination in zip((foreground.red, foreground.green, foreground.blue), background[:3])
    ) + (round(output_alpha * 255),)
