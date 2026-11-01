from dataclasses import dataclass
from enum import Enum

from .errors import DiagnosticRegistry, create_core_registry


class Severity(str, Enum):
    ERROR = "ERROR"
    WARNING = "WARNING"
    INFO = "INFO"


@dataclass(frozen=True)
class Diagnostic:
    severity: Severity
    code: str
    file: str
    line: int
    character: int
    description: str
    details: str | None = None
    category: str | None = None


class DiagnosticBag:
    def __init__(self, file: str = "<string>", *, registry: DiagnosticRegistry | None = None) -> None:
        self.items: list[Diagnostic] = []
        self.file = file
        self.registry = registry or create_core_registry()

    @property
    def has_errors(self) -> bool:
        return any(item.severity is Severity.ERROR for item in self.items)

    def add(
        self,
        severity: Severity,
        code: str,
        description: str,
        *,
        line: int = 1,
        character: int = 1,
        file: str | None = None,
        details: str | None = None,
    ) -> None:
        definition = self.registry.get(code)
        self.items.append(
            Diagnostic(
                severity,
                code,
                file or self.file,
                line,
                character,
                description,
                details if details is not None else definition.detail if definition else None,
                definition.category if definition else None,
            )
        )

    def error(self, code: str, description: str, **location: object) -> None:
        self.add(Severity.ERROR, code, description, **location)

    def warning(self, code: str, description: str, **location: object) -> None:
        self.add(Severity.WARNING, code, description, **location)

    def info(self, code: str, description: str, **location: object) -> None:
        self.add(Severity.INFO, code, description, **location)

    def register(self, definition, *, replace: bool = False) -> None:
        self.registry.register(definition, replace=replace)
