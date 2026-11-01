# API pública

La biblioteca `pixl_motorru` expone:

- `process_text(text, file)` para documentos estáticos.
- `process_text_svg(text, file)` para exigir salida SVG estática.
- `process_file(path)` para archivos estáticos.
- `process_animation(text, file)` para PIXL 1.2.
- `process_animation_file(path)` para animaciones desde disco.
- `Output::{Raster, Svg, Animation}` como resultado discriminado.
- `Diagnostics` y `Diagnostic` para errores y advertencias.
- `ErrorRegistry` para registrar definiciones adicionales sin modificar el
  núcleo.
- `write_raster_ppm`, `write_svg` y `write_animation_ppm` para exportar
  resultados sin dependencias de Python.

Los documentos con errores bloqueantes devuelven `None` como salida. Los
diagnósticos conservan el nombre del archivo, la línea y la columna de origen
para errores léxicos/sintácticos; algunos diagnósticos semánticos globales
siguen anclados al inicio del documento.

La exportación raster es PPM P6 (RGB; el alfa permanece disponible en la
estructura `Image`). SVG conserva sus canales RGBA mediante atributos de
opacidad.
