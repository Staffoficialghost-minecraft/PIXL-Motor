from __future__ import annotations

import tempfile
import webbrowser
from pathlib import Path

from .pipeline import process_animation, process_text


class MotorPyGui:
    def __init__(self, initial_text: str = "") -> None:
        try:
            import tkinter as tk
            from tkinter import filedialog, messagebox
        except ImportError as exc:
            raise RuntimeError("La GUI de PIXL MotorPY requiere tkinter") from exc

        self._tk = tk
        self._filedialog = filedialog
        self._messagebox = messagebox
        try:
            self.root = tk.Tk()
        except tk.TclError as exc:
            raise RuntimeError("No hay un entorno gráfico disponible para iniciar la GUI") from exc
        self.root.title("PIXL MotorPY")
        self.root.geometry("900x650")

        controls = tk.Frame(self.root)
        controls.pack(fill=tk.X, padx=8, pady=8)
        tk.Button(controls, text="Abrir", command=self.open_file).pack(side=tk.LEFT)
        tk.Button(controls, text="Procesar", command=self.process).pack(side=tk.LEFT, padx=4)
        self.status = tk.StringVar(value="Listo")
        tk.Label(controls, textvariable=self.status).pack(side=tk.LEFT, padx=8)

        body = tk.PanedWindow(self.root, orient=tk.HORIZONTAL, sashrelief=tk.RAISED)
        body.pack(fill=tk.BOTH, expand=True, padx=8, pady=(0, 8))
        editor_frame = tk.Frame(body)
        preview_frame = tk.Frame(body)
        body.add(editor_frame, stretch="always")
        body.add(preview_frame, stretch="always")

        self.editor = tk.Text(editor_frame, wrap=tk.NONE)
        self.editor.pack(fill=tk.BOTH, expand=True)
        self.editor.insert("1.0", initial_text)
        self.preview = tk.Label(preview_frame, text="Sin resultado", anchor=tk.CENTER)
        self.preview.pack(fill=tk.BOTH, expand=True)
        self.diagnostics = tk.Listbox(self.root, height=6)
        self.diagnostics.pack(fill=tk.X, padx=8, pady=(0, 8))

    def open_file(self) -> None:
        path = self._filedialog.askopenfilename(filetypes=[("PIXL", "*.pixl"), ("Todos", "*.*")])
        if not path:
            return
        try:
            with open(path, encoding="utf-8") as source_file:
                text = source_file.read()
        except (OSError, UnicodeError) as exc:
            self._messagebox.showerror("PIXL MotorPY", f"No se pudo leer el archivo: {exc}")
            return
        self.editor.delete("1.0", self._tk.END)
        self.editor.insert("1.0", text)
        self.status.set(path)

    def process(self) -> None:
        text = self.editor.get("1.0", self._tk.END)
        self.diagnostics.delete(0, self._tk.END)
        if "PIXL(1.2)" in text:
            result, diagnostics = process_animation(text, file="<GUI>")
            image = result[0].image if result else None
        else:
            result, diagnostics = process_text(text, file="<GUI>")
            if isinstance(result, str):
                self._open_svg_window(result)
                self.status.set("SVG abierto en otra ventana")
                self._show_diagnostics(diagnostics)
                return
            image = result
        self._show_diagnostics(diagnostics)
        if image is None:
            self.preview.configure(image="", text="Documento inválido")
            self.status.set("Error de procesamiento")
            return
        self._open_image_window(image)
        self.status.set(f"Render abierto en otra ventana: {image.width}x{image.height}")

    def _show_diagnostics(self, diagnostics) -> None:
        for diagnostic in diagnostics.items:
            self.diagnostics.insert(
                self._tk.END,
                f"[{diagnostic.severity}] {diagnostic.code} {diagnostic.line}:{diagnostic.character} {diagnostic.description}",
            )

    def _open_image_window(self, image) -> None:
        window = self._tk.Toplevel(self.root)
        window.title("PIXL MotorPY - Render")
        scale = max(1, min(32, 768 // max(image.width, image.height)))
        canvas = self._tk.Canvas(
            window,
            width=image.width * scale,
            height=image.height * scale,
            background="#808080",
            highlightthickness=0,
        )
        canvas.pack(expand=True, fill=self._tk.BOTH)
        for y in range(image.height):
            for x in range(image.width):
                red, green, blue, alpha = image.pixels[y * image.width + x]
                if alpha == 0:
                    color = "#808080" if (x + y) % 2 == 0 else "#A0A0A0"
                else:
                    color = "#%02x%02x%02x" % (red, green, blue)
                canvas.create_rectangle(
                    x * scale,
                    y * scale,
                    (x + 1) * scale,
                    (y + 1) * scale,
                    fill=color,
                    outline="",
                )

    @staticmethod
    def _open_svg_window(svg: str) -> None:
        handle = tempfile.NamedTemporaryFile(
            mode="w", suffix=".html", prefix="pixl-", encoding="utf-8", delete=False
        )
        try:
            handle.write(
                "<!doctype html><html><head><meta charset='utf-8'>"
                "<title>PIXL MotorPY - SVG</title>"
                "<style>"
                "html,body{margin:0;width:100%;height:100%;background:#303030;"
                "display:flex;align-items:center;justify-content:center;}"
                "svg{width:min(90vw,90vh);height:min(90vw,90vh);"
                "image-rendering:pixelated;image-rendering:crisp-edges;"
                "background-image:linear-gradient(45deg,#777 25%,transparent 25%),"
                "linear-gradient(-45deg,#777 25%,transparent 25%),"
                "linear-gradient(45deg,transparent 75%,#777 75%),"
                "linear-gradient(-45deg,transparent 75%,#777 75%);"
                "background-size:32px 32px;background-position:0 0,0 16px,16px -16px,-16px 0;}"
                "</style></head><body>"
                + svg
                + "</body></html>"
            )
            path = Path(handle.name)
        finally:
            handle.close()
        webbrowser.open(path.as_uri())

    def run(self) -> None:
        self.root.mainloop()


def launch_gui(initial_text: str = "") -> None:
    MotorPyGui(initial_text).run()
