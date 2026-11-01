import sys

from pixl.pipeline import process_animation_file, process_file
from pixl.gui import launch_gui


def main() -> int:
    if len(sys.argv) == 2 and sys.argv[1] in {"--help", "-h"}:
        print("Uso: python main.py [--animation] archivo.pixl | --gui")
        return 0
    if len(sys.argv) == 2 and sys.argv[1] == "--gui":
        try:
            launch_gui()
        except RuntimeError as error:
            print(f"No se pudo iniciar la GUI: {error}")
            return 1
        return 0
    if len(sys.argv) == 2:
        animation = False
        path = sys.argv[1]
    elif len(sys.argv) == 3 and sys.argv[1] == "--animation":
        animation = True
        path = sys.argv[2]
    else:
        print("Uso: python main.py [--animation] archivo.pixl | --gui")
        return 2
    result, diagnostics = (
        process_animation_file(path) if animation else process_file(path)
    )
    for diagnostic in diagnostics.items:
        print(f"[{diagnostic.severity}] {diagnostic.code} {diagnostic.file}:{diagnostic.line}:{diagnostic.character} {diagnostic.description}")
    if result is None:
        return 1
    if animation:
        print(f"Animación PIXL procesada correctamente: {len(result)} frames")
    elif isinstance(result, str):
        print("SVG procesado correctamente")
    else:
        print(f"Imagen RGBA {result.width}x{result.height} procesada correctamente")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
