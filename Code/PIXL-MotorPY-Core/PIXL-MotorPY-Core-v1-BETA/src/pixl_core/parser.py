from .diagnostics import DiagnosticBag
from .document import ColorReference, Command, Document, Frame, Layer, SourceLocation
from .lexer import Token


class Parser:
    def __init__(self, tokens: list[Token], diagnostics: DiagnosticBag) -> None:
        self.tokens = tokens
        self.diagnostics = diagnostics
        self.index = 0
        self.document = Document()
        self.current_layer: Layer | None = None
        self.current_frame: Frame | None = None
        self._singleton_declarations: set[str] = set()

    def parse(self) -> Document:
        while not self._at("EOF"):
            if self._at("NEWLINE"):
                self.index += 1
                continue
            self._statement()
        return self.document

    def _statement(self) -> None:
        start = self._current()
        if start.kind != "IDENT":
            self._syntax("se esperaba una declaración o comando")
            self._skip_line()
            return
        name = start.text
        self.index += 1
        if name == "GROUP":
            self._group(start)
        elif name in {"PIXL", "CANVAS", "TIPE", "RGBA", "HEX8", "LAYER", "FRAME"}:
            self._declaration(name, start)
        elif name in {"pixel", "rect", "fill", "inv", "circle", "ellipse", "polygon"}:
            self._command(name, start)
        else:
            self._syntax(f"elemento desconocido: {name}", start)
            self._skip_line()

    def _declaration(self, name: str, start: Token) -> None:
        if name == "GROUP":
            self._group(start)
            return
        if not self._accept("("):
            self._syntax("se esperaba '('", start)
            self._skip_line()
            return
        if self.current_frame is not None and name in {"CANVAS", "TIPE", "RGBA", "HEX8"}:
            self._syntax("las declaraciones globales deben aparecer antes del primer FRAME", start)
        values = self._values_until_close()
        if name in {"PIXL", "CANVAS", "TIPE"} and name in self._singleton_declarations:
            self.diagnostics.error(
                "PIXL021", f"declaración duplicada: {name}",
                line=start.line, character=start.character,
            )
        elif name == "PIXL" and len(values) == 1:
            self.document.version = str(values[0])
            self._singleton_declarations.add(name)
        elif name == "CANVAS" and len(values) == 2 and all(isinstance(v, int) for v in values):
            self.document.canvas = (values[0], values[1])
            self._singleton_declarations.add(name)
        elif name == "TIPE" and len(values) == 1:
            self.document.tipe = str(values[0])
            self._singleton_declarations.add(name)
        elif name == "RGBA" and len(values) == 1:
            self._set_color_system(f"RGBA({values[0]})", start)
        elif name == "HEX8" and not values:
            self._set_color_system("HEX8", start)
        elif name == "FRAME" and len(values) == 2 and all(isinstance(v, int) for v in values):
            frame = Frame(values[0], values[1], SourceLocation(start.line, start.character))
            self.document.frames.append(frame)
            self.current_frame = frame
            self.current_layer = None
        elif name == "LAYER" and len(values) == 1 and isinstance(values[0], int):
            layer = Layer(values[0], SourceLocation(start.line, start.character))
            if self.current_frame is not None:
                self.current_frame.layers.append(layer)
            else:
                self.document.layers.append(layer)
            self.current_layer = layer
        else:
            self._syntax(f"argumentos inválidos para {name}", start)

    def _set_color_system(self, value: str, token: Token) -> None:
        if self.document.color_system is not None:
            self.diagnostics.error(
                "PIXL020", "solo se puede declarar un sistema de color por documento",
                line=token.line, character=token.character,
            )
        else:
            self.document.color_system = value

    def _group(self, start: Token) -> None:
        if not self._accept("="):
            self._syntax("se esperaba '=' después de GROUP", start)
            self._skip_line()
            return
        name = self._current()
        self.index += 1
        if name.kind not in {"IDENT", "HEX"} or name.text.lower() != name.text:
            self._syntax("el nombre del grupo debe estar en minúsculas", name)
        if not self._accept("("):
            self._syntax("se esperaba '(' en la declaración GROUP", name)
            self._skip_line()
            return
        values = self._values_until_close()
        if not (len(values) == 1 and isinstance(values[0], str)) and len(values) != 4:
            self._syntax("GROUP requiere un único color", start)
            return
        ref = ColorReference(values[0] if len(values) == 1 else tuple(values), SourceLocation(name.line, name.character))
        target = (
            self.current_layer.groups if self.current_layer
            else self.current_frame.groups if self.current_frame
            else self.document.global_groups
        )
        if name.text in target:
            self.diagnostics.error(
                "PIXL022", f"identificador de grupo duplicado: {name.text}",
                line=name.line, character=name.character,
            )
        else:
            target[name.text] = ref

    def _command(self, name: str, start: Token) -> None:
        if self.current_layer is None:
            self._syntax("los comandos gráficos deben estar dentro de LAYER", start)
            self._skip_line()
            return
        if not self._accept("("):
            self._syntax("se esperaba '('", start)
            self._skip_line()
            return
        values = self._values_until_close(keep_ranges=True)
        self.current_layer.commands.append(
            Command(name, tuple(values), SourceLocation(start.line, start.character))
        )

    def _values_until_close(self, keep_ranges: bool = False) -> list[object]:
        values: list[object] = []
        separator_required = False
        while not self._at("EOF") and not self._at("NEWLINE") and not self._at(")"):
            token = self._current()
            if token.kind == ",":
                if not separator_required:
                    self._syntax("separador inesperado", token)
                separator_required = False
                self.index += 1
                continue
            if token.kind == "-" and keep_ranges and separator_required:
                separator_required = False
                self.index += 1
                values.append("-")
                continue
            if token.kind == "-" and not separator_required:
                next_token = self.tokens[self.index + 1]
                if next_token.kind == "NUMBER":
                    number = -float(next_token.text) if "." in next_token.text else -int(next_token.text)
                    values.append(number)
                    self.index += 2
                    separator_required = True
                    continue
            if token.kind in {"NUMBER", "IDENT", "HEX"}:
                if separator_required:
                    self._syntax("se esperaba ',' entre argumentos", token)
                if token.kind == "NUMBER":
                    values.append(float(token.text) if "." in token.text else int(token.text))
                else:
                    values.append(token.text)
                separator_required = True
                self.index += 1
                continue
            if token.kind == "-":
                if keep_ranges:
                    values.append("-")
                else:
                    self._syntax("'-' solo es válido en rangos de comandos gráficos", token)
                self.index += 1
                continue
            if token.kind != "INVALID":
                self._syntax("token inválido", token)
            self.index += 1
        if self._at(")") and not separator_required and values:
            self._syntax("se esperaba un argumento después de ','", self._current())
        if not self._accept(")"):
            self._syntax("estructura incompleta: se esperaba ')'", self._current())
        return self._combine_ranges(values)

    @staticmethod
    def _combine_ranges(values: list[object]) -> list[object]:
        result: list[object] = []
        index = 0
        while index < len(values):
            if index + 2 < len(values) and isinstance(values[index], int) and values[index + 1] == "-" and isinstance(values[index + 2], int):
                result.append((values[index], values[index + 2]))
                index += 3
            else:
                result.append(values[index])
                index += 1
        return result

    def _accept(self, kind: str) -> bool:
        if self._at(kind):
            self.index += 1
            return True
        return False

    def _at(self, kind: str) -> bool:
        return self._current().kind == kind

    def _current(self) -> Token:
        return self.tokens[self.index]

    def _skip_line(self) -> None:
        while not self._at("EOF") and not self._at("NEWLINE"):
            self.index += 1

    def _syntax(self, message: str, token: Token | None = None) -> None:
        token = token or self._current()
        self.diagnostics.error("PIXL001", message, line=token.line, character=token.character)
