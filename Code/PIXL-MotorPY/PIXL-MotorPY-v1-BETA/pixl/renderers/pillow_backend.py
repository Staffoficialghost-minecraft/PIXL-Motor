from ..renderer import RenderedImage


class PillowBackend:
    """Optional Pillow adapter; PIXL parsing and rendering do not depend on it."""

    def to_image(self, image: RenderedImage):
        try:
            from PIL import Image
        except ImportError as exc:
            raise RuntimeError("el backend Pillow requiere instalar Pillow") from exc
        result = Image.new("RGBA", (image.width, image.height), (0, 0, 0, 0))
        result.putdata(image.pixels)
        return result
