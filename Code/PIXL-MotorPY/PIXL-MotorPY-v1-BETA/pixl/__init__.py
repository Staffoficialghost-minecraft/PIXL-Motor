"""PIXL MotorPY parser, validator, resolver, and renderer."""

from .diagnostics import Diagnostic, DiagnosticBag, Severity
from .document import Animation, AnimationFrame, Document
from .errors import DiagnosticRegistry, ErrorDefinition, create_core_registry
from .lexer import Lexer
from .parser import Parser
from .pipeline import process_animation, process_animation_file, process_file, process_text, process_text_svg
from .renderer import RenderedImage, render, render_svg

__all__ = [
    "Diagnostic",
    "DiagnosticBag",
    "Document",
    "DiagnosticRegistry",
    "ErrorDefinition",
    "Animation",
    "AnimationFrame",
    "Lexer",
    "Parser",
    "RenderedImage",
    "Severity",
    "process_text",
    "process_text_svg",
    "process_animation",
    "process_animation_file",
    "process_file",
    "render",
    "render_svg",
    "create_core_registry",
]
