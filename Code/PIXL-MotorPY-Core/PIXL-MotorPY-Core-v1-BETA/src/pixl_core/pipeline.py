from __future__ import annotations

from pathlib import Path

from .diagnostics import DiagnosticBag
from .document import Document
from .ir import ProcessedAnimation, ProcessedDocument
from .lexer import Lexer
from .normalize import normalize_animation, normalize_document
from .parser import Parser
from .validator import Validator


def parse_text(
    text: str, *, file: str = "<string>"
) -> tuple[Document, DiagnosticBag]:
    diagnostics = DiagnosticBag(file)
    document = Parser(Lexer(diagnostics).tokenize(text), diagnostics).parse()
    return document, diagnostics


def process_text(
    text: str, *, file: str = "<string>"
) -> tuple[ProcessedDocument | None, DiagnosticBag]:
    document, diagnostics = parse_text(text, file=file)
    Validator().validate(document, diagnostics)
    if document.version == "1.2":
        diagnostics.error("PIXL054", "PIXL(1.2) requiere process_animation", line=1, character=1)
    if diagnostics.has_errors:
        return None, diagnostics
    return normalize_document(document, diagnostics), diagnostics


def process_animation(
    text: str, *, file: str = "<string>"
) -> tuple[ProcessedAnimation | None, DiagnosticBag]:
    document, diagnostics = parse_text(text, file=file)
    Validator().validate(document, diagnostics)
    if document.version != "1.2":
        diagnostics.error("PIXL058", "process_animation requiere PIXL(1.2)", line=1, character=1)
    if diagnostics.has_errors:
        return None, diagnostics
    return normalize_animation(document, diagnostics), diagnostics


def parse_file(
    path: str | Path, *, file: str | None = None
) -> tuple[Document | None, DiagnosticBag]:
    source, diagnostics = _read_file(path, file=file)
    if source is None:
        return None, diagnostics
    return parse_text(source, file=diagnostics.file)


def process_file(
    path: str | Path, *, file: str | None = None
) -> tuple[ProcessedDocument | None, DiagnosticBag]:
    source, diagnostics = _read_file(path, file=file)
    if source is None:
        return None, diagnostics
    return process_text(source, file=diagnostics.file)


def process_animation_file(
    path: str | Path, *, file: str | None = None
) -> tuple[ProcessedAnimation | None, DiagnosticBag]:
    source, diagnostics = _read_file(path, file=file)
    if source is None:
        return None, diagnostics
    return process_animation(source, file=diagnostics.file)


def _read_file(path: str | Path, *, file: str | None):
    path_object = Path(path)
    display_name = file or str(path_object)
    diagnostics = DiagnosticBag(display_name)
    try:
        return path_object.read_text(encoding="utf-8"), diagnostics
    except FileNotFoundError:
        diagnostics.error("PIXL060", f"No se encontró el archivo PIXL: {path_object}")
    except IsADirectoryError:
        diagnostics.error("PIXL061", f"La ruta no es un archivo PIXL: {path_object}")
    except (OSError, UnicodeError) as error:
        diagnostics.error("PIXL062", f"No se pudo leer el archivo PIXL: {error}")
    return None, diagnostics
