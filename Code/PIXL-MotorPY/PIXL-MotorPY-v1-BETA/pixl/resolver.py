from .diagnostics import DiagnosticBag
from .document import ColorReference, ColorValue, Document, SourceLocation


class Resolver:
    def resolve_color(
        self,
        value: object,
        document: Document,
        local_groups: dict[str, ColorReference],
        diagnostics: DiagnosticBag,
        location: SourceLocation,
    ) -> ColorValue | None:
        if isinstance(value, ColorReference):
            value = value.value
        if not isinstance(value, str):
            return self._components(value, document)
        if value in local_groups:
            return self._literal(local_groups[value].value, document, diagnostics, location)
        if value in document.global_groups:
            return self._literal(document.global_groups[value].value, document, diagnostics, location)
        return self._literal(value, document, diagnostics, location)

    def _literal(self, value: object, document: Document, diagnostics: DiagnosticBag, location: SourceLocation) -> ColorValue | None:
        if isinstance(value, tuple):
            return self._components(value, document)
        if document.color_system == "HEX8" and len(value) == 8 and all(c in "0123456789abcdefABCDEF" for c in value):
            return ColorValue(*(int(value[index:index + 2], 16) for index in range(0, 8, 2)))
        diagnostics.warning("PIXL040", f"grupo o color desconocido: {value}; se usará transparencia", line=location.line, character=location.character)
        return ColorValue(0, 0, 0, 0)

    @staticmethod
    def _components(value: object, document: Document) -> ColorValue | None:
        if not isinstance(value, tuple) or len(value) != 4:
            return None
        if document.color_system == "RGBA(1)":
            return ColorValue(*(round(float(component) * 255) for component in value))
        return ColorValue(*(round(float(component)) for component in value))
