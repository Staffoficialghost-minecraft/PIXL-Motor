from .diagnostics import DiagnosticBag
from .document import Command, Document, Frame


class Validator:
    def validate(self, document: Document, diagnostics: DiagnosticBag) -> None:
        if document.version not in {"1.1", "1.2"}:
            diagnostics.error("PIXL010", "solo PIXL(1.1) y PIXL(1.2) están implementados")
        if document.canvas is None:
            diagnostics.error("PIXL011", "falta la declaración CANVAS")
        elif document.canvas[0] <= 0 or document.canvas[1] <= 0:
            diagnostics.error("PIXL012", "las dimensiones del lienzo deben ser positivas")
        if document.tipe not in {"bit", "vec"}:
            diagnostics.error("PIXL013", "TIPE debe ser bit o vec")
        if document.color_system not in {"HEX8", "RGBA(1)", "RGBA(2)"}:
            diagnostics.error("PIXL014", "falta un sistema de color válido")
        if document.version == "1.2":
            if document.layers:
                diagnostics.error("PIXL055", "PIXL(1.2) no permite capas fuera de FRAME")
            if not document.frames:
                diagnostics.error("PIXL056", "PIXL(1.2) requiere al menos un FRAME")
            if document.tipe == "vec":
                diagnostics.error("PIXL063", "la exportación SVG de animaciones no está implementada")
            self._validate_frames(document, diagnostics)
            for name, reference in document.global_groups.items():
                self._group_color(name, reference.value, document, diagnostics, reference.location)
            return
        if document.frames:
            diagnostics.error("PIXL057", "FRAME solo está disponible en PIXL(1.2)")
        if not document.layers:
            diagnostics.error("PIXL015", "el documento debe declarar al menos una capa")
        seen_layers: set[int] = set()
        for layer in document.layers:
            if layer.number <= 0:
                diagnostics.error("PIXL016", "el número de capa debe ser positivo", line=layer.location.line, character=layer.location.character)
            if layer.number in seen_layers:
                diagnostics.error("PIXL017", f"capa duplicada: {layer.number}", line=layer.location.line, character=layer.location.character)
            seen_layers.add(layer.number)
            for command in layer.commands:
                self._command(command, document, diagnostics)
            for name, reference in layer.groups.items():
                self._group_color(name, reference.value, document, diagnostics, reference.location)
        for name, reference in document.global_groups.items():
            self._group_color(name, reference.value, document, diagnostics, reference.location)

    def _validate_frames(self, document: Document, diagnostics: DiagnosticBag) -> None:
        seen: set[int] = set()
        previous = 0
        for frame in document.frames:
            if frame.number <= 0 or frame.number <= previous:
                diagnostics.error("PIXL050", "los frames deben tener números positivos estrictamente crecientes", line=frame.location.line, character=frame.location.character)
            if frame.number in seen:
                diagnostics.error("PIXL051", "número de frame duplicado", line=frame.location.line, character=frame.location.character)
            if frame.duration_ms <= 0:
                diagnostics.error("PIXL052", "la duración debe ser un entero positivo en milisegundos", line=frame.location.line, character=frame.location.character)
            seen.add(frame.number)
            previous = frame.number
            if not frame.layers:
                diagnostics.error("PIXL053", "los frames vacíos no están permitidos", line=frame.location.line, character=frame.location.character)
            seen_layers: set[int] = set()
            for layer in frame.layers:
                if layer.number <= 0:
                    diagnostics.error("PIXL016", "el número de capa debe ser positivo", line=layer.location.line, character=layer.location.character)
                if layer.number in seen_layers:
                    diagnostics.error("PIXL017", f"capa duplicada: {layer.number}", line=layer.location.line, character=layer.location.character)
                seen_layers.add(layer.number)
                for command in layer.commands:
                    self._command(command, document, diagnostics)
                for name, reference in layer.groups.items():
                    self._group_color(name, reference.value, document, diagnostics, reference.location)
            for name, reference in frame.groups.items():
                self._group_color(name, reference.value, document, diagnostics, reference.location)

    def _group_color(self, name: str, value: object, document: Document, diagnostics: DiagnosticBag, location) -> None:
        if document.color_system == "HEX8":
            valid = isinstance(value, str) and len(value) == 8 and all(c in "0123456789abcdefABCDEF" for c in value)
        else:
            limit = 1 if document.color_system == "RGBA(1)" else 255
            valid = isinstance(value, tuple) and len(value) == 4 and all(
                isinstance(component, (int, float)) and 0 <= component <= limit for component in value
            )
        if not valid:
            diagnostics.error("PIXL033", f"color inválido en GROUP: {name}", line=location.line, character=location.character)

    def _command(self, command: Command, document: Document, diagnostics: DiagnosticBag) -> None:
        width, height = document.canvas or (0, 0)
        args = command.arguments
        if command.name == "inv":
            valid = len(args) == 2 and all(isinstance(value, int) for value in args)
            self._coordinates(valid, args, width, height, command, diagnostics)
            return
        if command.name == "pixel":
            valid = len(args) == (6 if document.color_system and document.color_system.startswith("RGBA") else 3)
            valid = valid and isinstance(args[0], int) and isinstance(args[1], int)
            self._coordinates(valid, args[:2], width, height, command, diagnostics)
            if valid:
                self._color(args[2:], document, command, diagnostics)
            return
        if command.name in {"rect", "fill"}:
            color_end = 6 if document.color_system and document.color_system.startswith("RGBA") else 3
            valid = len(args) >= color_end
            if command.name == "fill" and len(args) != color_end:
                valid = False
            valid = valid and isinstance(args[0], tuple) and isinstance(args[1], tuple)
            self._range_coordinates(valid, args[:2], width, height, command, diagnostics)
            if valid:
                color_end = 3 if document.color_system == "HEX8" else 6
                self._color(args[2:color_end], document, command, diagnostics)
            if command.name == "rect" and len(args) > (3 if document.color_system == "HEX8" else 6):
                self._style(args[3 if document.color_system == "HEX8" else 6:], command, diagnostics)
            return
        if command.name in {"circle", "ellipse", "polygon"}:
            self._shape(command, document, diagnostics)

    @staticmethod
    def _style(values, command, diagnostics) -> None:
        if not values or values[0] not in {"fill", "outline"}:
            diagnostics.error("PIXL034", "se esperaba fill u outline", line=command.location.line, character=command.location.character)
            return
        if values[0] == "fill":
            if len(values) != 1:
                diagnostics.error("PIXL035", "fill no admite grosor ni posición", line=command.location.line, character=command.location.character)
        elif len(values) not in {1, 3} or (len(values) == 3 and (not isinstance(values[1], int) or values[1] <= 0 or values[2] not in {"i", "c", "e"})):
            diagnostics.error("PIXL035", "outline requiere grosor positivo y posición i, c o e", line=command.location.line, character=command.location.character)

    def _shape(self, command, document, diagnostics) -> None:
        args = command.arguments
        try:
            mode = max(index for index, value in enumerate(args) if value in {"fill", "outline"})
        except ValueError:
            diagnostics.error("PIXL034", "la figura requiere modo fill u outline", line=command.location.line, character=command.location.character)
            return
        color_count = 1 if document.color_system == "HEX8" else 4
        color_start = mode - color_count
        if color_start < 0:
            diagnostics.error("PIXL032", "color faltante", line=command.location.line, character=command.location.character)
            return
        shape_valid = True
        if command.name == "circle" and color_start != 3:
            diagnostics.error("PIXL036", "circle requiere x, y y radius antes del color", line=command.location.line, character=command.location.character)
            shape_valid = False
        elif command.name == "ellipse" and color_start != 4:
            diagnostics.error("PIXL036", "ellipse requiere cuatro límites antes del color", line=command.location.line, character=command.location.character)
            shape_valid = False
        elif command.name == "polygon" and (color_start < 6 or color_start % 2 or not all(isinstance(value, int) for value in args[:color_start])):
            diagnostics.error("PIXL036", "polygon requiere al menos tres vértices enteros", line=command.location.line, character=command.location.character)
            shape_valid = False
        if command.name == "circle" and shape_valid and (not all(isinstance(value, int) for value in args[:3]) or args[2] <= 0):
            diagnostics.error("PIXL037", "circle requiere un radio entero positivo", line=command.location.line, character=command.location.character)
        if command.name == "ellipse" and shape_valid and (not all(isinstance(value, int) for value in args[:4]) or args[0] >= args[2] or args[1] >= args[3]):
            diagnostics.error("PIXL037", "ellipse requiere límites crecientes", line=command.location.line, character=command.location.character)
        if command.name == "polygon" and shape_valid:
            points = list(zip(args[:color_start:2], args[1:color_start:2]))
            if len(set(points)) != len(points):
                diagnostics.error("PIXL038", "polygon no admite vértices repetidos", line=command.location.line, character=command.location.character)
            area = sum(
                first[0] * second[1] - second[0] * first[1]
                for first, second in zip(points, points[1:] + points[:1])
            )
            if area == 0:
                diagnostics.error("PIXL039", "polygon degenerado: vértices colineales", line=command.location.line, character=command.location.character)
        self._color(args[color_start:mode], document, command, diagnostics)
        self._style(args[mode:], command, diagnostics)

    @staticmethod
    def _coordinates(valid: bool, args: tuple[object, ...], width: int, height: int, command: Command, diagnostics: DiagnosticBag) -> None:
        if not valid or len(args) != 2 or not (1 <= args[0] <= width and 1 <= args[1] <= height):
            diagnostics.error("PIXL030", "coordenadas fuera de rango o argumentos inválidos", line=command.location.line, character=command.location.character)

    @staticmethod
    def _range_coordinates(valid: bool, args: tuple[object, ...], width: int, height: int, command: Command, diagnostics: DiagnosticBag) -> None:
        if not valid or any(not isinstance(value, tuple) or value[0] > value[1] or value[0] < 1 or value[1] > limit for value, limit in zip(args, (width, height))):
            diagnostics.error("PIXL031", "rango de coordenadas inválido", line=command.location.line, character=command.location.character)

    @staticmethod
    def _color(values: tuple[object, ...], document: Document, command: Command, diagnostics: DiagnosticBag) -> None:
        if document.color_system == "HEX8":
            valid = len(values) == 1 and isinstance(values[0], str) and (
                values[0].islower() or (
                    len(values[0]) == 8 and all(c in "0123456789abcdefABCDEF" for c in values[0])
                )
            )
        else:
            limit = 1 if document.color_system == "RGBA(1)" else 255
            valid = (len(values) == 1 and isinstance(values[0], str)) or (
                len(values) == 4 and all(isinstance(value, (int, float)) and 0 <= value <= limit for value in values)
            )
        if not valid:
            diagnostics.error("PIXL032", "color inválido para el sistema declarado", line=command.location.line, character=command.location.character)
