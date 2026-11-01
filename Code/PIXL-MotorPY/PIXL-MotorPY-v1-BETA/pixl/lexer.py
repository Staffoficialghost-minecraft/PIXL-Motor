from dataclasses import dataclass
import re

from .diagnostics import DiagnosticBag


@dataclass(frozen=True)
class Token:
    kind: str
    text: str
    line: int
    character: int
    end_character: int


class Lexer:
    _pattern = re.compile(
        r"(?P<space>[ \t]+)|(?P<hex>[0-9A-Fa-f]{8})|"
        r"(?P<number>[0-9]+(?:\.[0-9]+)?)|(?P<ident>[A-Za-z_][A-Za-z0-9_]*)|"
        r"(?P<symbol>[(),=.-])|(?P<other>\S)"
    )

    def __init__(self, diagnostics: DiagnosticBag | None = None) -> None:
        self.diagnostics = diagnostics

    def tokenize(self, text: str) -> list[Token]:
        tokens: list[Token] = []
        for line_number, line in enumerate(text.splitlines(), 1):
            source_line = line.split("#", 1)[0]
            for match in self._pattern.finditer(source_line):
                kind = match.lastgroup
                if kind == "space":
                    continue
                value = match.group()
                if kind == "symbol":
                    kind = value
                elif kind == "hex":
                    kind = "HEX"
                elif kind == "number":
                    kind = "NUMBER"
                elif kind == "ident":
                    kind = "IDENT"
                else:
                    kind = "INVALID"
                start = match.start() + 1
                tokens.append(Token(kind, value, line_number, start, match.end()))
                if kind == "INVALID" and self.diagnostics is not None:
                    self.diagnostics.error(
                        "PIXL002",
                        f"carácter inesperado: {value!r}",
                        line=line_number,
                        character=start,
                    )
            tokens.append(Token("NEWLINE", "", line_number, len(source_line) + 1, len(source_line) + 1))
        tokens.append(Token("EOF", "", max(1, text.count("\n") + 1), 1, 1))
        return tokens
