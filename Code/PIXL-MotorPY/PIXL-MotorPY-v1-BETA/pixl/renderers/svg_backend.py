from html import escape

from ..document import PixelState
from ..render_model import RenderScene, SceneCell


class SvgBackend:
    """Converts a resolved render scene to SVG without parsing PIXL or using Pillow."""

    def to_svg(self, scene: RenderScene) -> str:
        parts = [
            '<svg xmlns="http://www.w3.org/2000/svg" '
            f'width="{scene.width}" height="{scene.height}" '
            f'viewBox="0 0 {scene.width} {scene.height}">'
        ]
        for layer in scene.layers:
            parts.append(f'<g data-layer="{layer.number}">')
            parts.extend(self._layer_rects(layer.cells, layer.width, layer.height))
            parts.append("</g>")
        parts.append("</svg>")
        return "".join(parts)

    def _layer_rects(
        self, cells: tuple[SceneCell, ...], width: int, height: int
    ) -> list[str]:
        elements: list[str] = []
        for y in range(1, height + 1):
            x = 1
            while x <= width:
                cell = cells[(y - 1) * width + x - 1]
                if not self._is_visible(cell):
                    x += 1
                    continue
                end = x + 1
                while end <= width and cells[(y - 1) * width + end - 1] == cell:
                    end += 1
                elements.append(self._rect(x, y, end - x, cell))
                x = end
        return elements

    @staticmethod
    def _is_visible(cell: SceneCell) -> bool:
        return (
            cell.state is PixelState.DECLARED
            and cell.color is not None
            and cell.color.alpha > 0
        )

    @staticmethod
    def _rect(x: int, y: int, width: int, cell: SceneCell) -> str:
        color = cell.color
        assert color is not None
        attributes = (
            f'x="{escape(str(x - 1), quote=True)}" '
            f'y="{escape(str(y - 1), quote=True)}" '
            f'width="{escape(str(width), quote=True)}" height="1" '
            f'fill="#{color.red:02X}{color.green:02X}{color.blue:02X}"'
        )
        if color.alpha < 255:
            attributes += f' fill-opacity="{color.alpha / 255:g}"'
        return f"<rect {attributes}/>"
