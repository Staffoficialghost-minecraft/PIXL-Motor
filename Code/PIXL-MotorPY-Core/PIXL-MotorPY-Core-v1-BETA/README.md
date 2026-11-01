# PIXL MotorPY-Core

Biblioteca BETA independiente para analizar y normalizar documentos PIXL.

El paquete de distribución es `pixl-motorpy-core` y el paquete de importación
es `pixl_core`.

```powershell
python -m pip install pixl-motorpy-core
```

```python
import pixl_core

document, diagnostics = pixl_core.process_text(text)
animation, diagnostics = pixl_core.process_animation(animation_text)
```

MotorPY-Core contiene lexer, parser, validator, diagnósticos, resolución de
grupos y una IR normalizada. No contiene GUI, CLI, Pillow, Tkinter,
rasterización ni exportación de imágenes.

La IR es experimental durante la versión BETA. `ProcessedDocument` y
`ProcessedAnimation` contienen operaciones normalizadas, colores RGBA de
8 bits, capas, frames, estilos y ubicaciones de origen.

## Desarrollo local

```powershell
python -m pytest tests -q
python -m build
```

El paquete no se publica automáticamente. La licencia declarada es
`Apache-2.0`. El texto íntegro oficial deberá incorporarse desde la fuente
del proyecto cuando `Motors` se suba a GitHub; este árbol local no contiene
todavía el archivo `LICENSE`.
