from dataclasses import dataclass
from threading import RLock


@dataclass(frozen=True)
class ErrorDefinition:
    code: str
    category: str
    description: str
    detail: str = ""


class DiagnosticRegistry:
    def __init__(self, definitions: tuple[ErrorDefinition, ...] = ()) -> None:
        self._definitions: dict[str, ErrorDefinition] = {}
        self._lock = RLock()
        for definition in definitions:
            self.register(definition)

    def register(self, definition: ErrorDefinition, *, replace: bool = False) -> None:
        with self._lock:
            if definition.code in self._definitions and not replace:
                raise ValueError(f"diagnostic code already registered: {definition.code}")
            self._definitions[definition.code] = definition

    def get(self, code: str) -> ErrorDefinition | None:
        with self._lock:
            return self._definitions.get(code)

    def definitions(self) -> tuple[ErrorDefinition, ...]:
        with self._lock:
            return tuple(self._definitions.values())


CORE_DIAGNOSTICS = (
    ErrorDefinition("PIXL001", "syntax", "Error de sintaxis", "La instrucción no cumple la gramática PIXL."),
    ErrorDefinition("PIXL002", "lexical", "Carácter inesperado", "El lexer encontró un carácter no válido."),
    ErrorDefinition("PIXL010", "version", "Versión PIXL no soportada", "La versión del lenguaje no está implementada."),
    ErrorDefinition("PIXL011", "configuration", "Falta CANVAS", "El documento debe declarar un lienzo."),
    ErrorDefinition("PIXL012", "configuration", "Dimensiones inválidas", "El lienzo debe tener dimensiones positivas."),
    ErrorDefinition("PIXL013", "configuration", "Tipo de renderizado no soportado", "Los tipos válidos son TIPE(bit) y TIPE(vec)."),
    ErrorDefinition("PIXL014", "configuration", "Sistema de color inválido", "Debe declararse un sistema de color válido."),
    ErrorDefinition("PIXL016", "layer", "Número de capa inválido", "El número de capa debe ser positivo."),
    ErrorDefinition("PIXL017", "layer", "Capa duplicada", "No se permiten capas duplicadas en el mismo ámbito."),
    ErrorDefinition("PIXL020", "configuration", "Sistema de color duplicado", "Solo puede declararse un sistema de color."),
    ErrorDefinition("PIXL021", "configuration", "Declaración duplicada", "La declaración solo puede aparecer una vez."),
    ErrorDefinition("PIXL022", "group", "Grupo duplicado", "El identificador de grupo ya existe en el ámbito."),
    ErrorDefinition("PIXL030", "coordinates", "Coordenadas inválidas", "Las coordenadas están fuera del lienzo."),
    ErrorDefinition("PIXL031", "coordinates", "Rango inválido", "El rango no es válido para el lienzo."),
    ErrorDefinition("PIXL032", "color", "Color inválido", "El color no corresponde al sistema declarado."),
    ErrorDefinition("PIXL033", "group", "Color de grupo inválido", "La declaración GROUP contiene un color inválido."),
    ErrorDefinition("PIXL040", "group", "Grupo desconocido", "Se utilizará transparencia como fallback."),
    ErrorDefinition("PIXL050", "animation", "Numeración de frame inválida", "Los frames deben aumentar estrictamente."),
    ErrorDefinition("PIXL052", "animation", "Duración inválida", "La duración debe ser positiva."),
    ErrorDefinition("PIXL053", "animation", "Frame vacío", "Los frames deben contener capas."),
    ErrorDefinition("PIXL054", "animation", "API incompatible", "Los documentos 1.2 requieren process_animation."),
    ErrorDefinition("PIXL055", "animation", "Capas estáticas en 1.2", "PIXL(1.2) no permite capas fuera de FRAME."),
    ErrorDefinition("PIXL056", "animation", "Faltan frames", "PIXL(1.2) requiere al menos un FRAME."),
    ErrorDefinition("PIXL057", "version", "FRAME no permitido", "FRAME solo está disponible en PIXL(1.2)."),
    ErrorDefinition("PIXL058", "animation", "Documento no animado", "process_animation requiere PIXL(1.2)."),
    ErrorDefinition("PIXL038", "geometry", "Vértices repetidos", "Un polígono no puede repetir vértices."),
    ErrorDefinition("PIXL039", "geometry", "Polígono degenerado", "Los vértices no pueden ser todos colineales."),
    ErrorDefinition("PIXL063", "animation", "SVG animado no soportado", "La exportación SVG de animaciones no está implementada."),
    ErrorDefinition("PIXL060", "io", "Archivo no encontrado", "No se pudo encontrar el archivo PIXL."),
    ErrorDefinition("PIXL061", "io", "Ruta inválida", "La ruta no corresponde a un archivo PIXL."),
    ErrorDefinition("PIXL062", "io", "Error de lectura", "No se pudo leer el archivo PIXL."),
    ErrorDefinition("418", "internal", "El servidor es un árbol.", "Diagnóstico humorístico reservado."),
    ErrorDefinition("419", "internal", "El servidor se volvió un hongo.", "Diagnóstico humorístico reservado."),
    ErrorDefinition("420", "internal", "El servidor echó raíces.", "Diagnóstico humorístico reservado."),
)


def create_core_registry() -> DiagnosticRegistry:
    return DiagnosticRegistry(CORE_DIAGNOSTICS)
