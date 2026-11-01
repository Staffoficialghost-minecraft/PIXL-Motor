# PIXL MotorPY

Esta implementación de PIXL MotorPY procesa documentos PIXL 1.1 y 1.2 en cuatro etapas:

1. `Lexer` conserva tokens y ubicaciones.
2. `Parser` construye la representación interna sin dibujar.
3. `Validator` comprueba declaraciones, coordenadas, colores y comandos.
4. `Resolver` y `renderer` resuelven grupos y producen una imagen RGBA.

La especificación común está en
[`SPECIFICATION.md`](./SPECIFICATION.md).
La guía práctica para escribir archivos PIXL está en
[`structure.md`](./structure.md).

El renderer usa una imagen interna independiente de Pillow. `pixl.renderers.PillowBackend`
es un adaptador opcional para convertirla a `PIL.Image`; Pillow no se importa durante
el análisis ni la validación.

Instalación independiente:

```bash
python -m pip install .
```

La instalación no requiere Mythree ni Pillow. Pillow solo es necesario para el
adaptador opcional `PillowBackend`. `TIPE(vec)` genera SVG automáticamente.

Ejecutar pruebas:

```bash
python -m pytest tests -q
```

Procesar un archivo estático:

```bash
python main.py dibujo.pixl
```

Procesar una animación PIXL 1.2:

```bash
python main.py --animation animacion.pixl
```

La API equivalente es `process_file()` para documentos estáticos y
`process_animation_file()` para documentos 1.2.

Los grupos duplicados producen `PIXL022`; los grupos desconocidos producen
`PIXL040` y transparencia. Se admiten ámbitos global, de frame y de `LAYER`,
con prioridad `LAYER > FRAME > GLOBAL`.

El Lexer admite comentarios de línea con `#`, conserva los saltos de línea y
produce `PIXL002` para caracteres inesperados. El Parser produce `PIXL001` para
errores sintácticos y recupera el análisis al final de la línea cuando es posible.
Los errores léxicos o sintácticos bloquean el renderizado.

Para obtener SVG mediante el pipeline validado:

```python
from pixl import process_text_svg

svg, diagnostics = process_text_svg(text, file="dibujo.pixl")
```

El backend SVG recibe únicamente la escena intermedia resuelta; no analiza PIXL,
no valida sintaxis y no importa Pillow.

## GUI y diagnósticos extensibles

MotorPY incluye una GUI mínima para abrir y procesar archivos, visualizar el
resultado raster y consultar diagnósticos:

```bash
pixl --gui
```

La GUI usa `tkinter` de forma opcional y no añade una dependencia de renderizado.
El catálogo de diagnósticos está disponible mediante `ErrorDefinition` y
`DiagnosticRegistry`. Los renderizadores pueden registrar códigos propios, por
ejemplo `RENDER001`, sin modificar el Core. Los códigos humorísticos internos
418, 419 y 420 están reservados.

## Estado de PIXL MotorPY

La implementación actual es la referencia funcional `vPY 0.1.0`. Están
implementados `PIXL(1.1)`, `PIXL(1.2)`,
`CANVAS`, `TIPE(bit)`, `TIPE(vec)`, `HEX8`, `RGBA(1)`, `RGBA(2)`, `LAYER`, `GROUP` global,
de frame y de capa, `pixel`, `inv`, `rect`, `fill`, `circle`, `ellipse`,
`polygon`, contornos raster, SVG, diagnósticos y la CLI básica.

`PIXL(1.2)` usa `FRAME(numero, duracion_ms)`, con números positivos
estrictamente crecientes, duraciones enteras positivas en milisegundos,
frames no vacíos e independientes. Los grupos globales deben declararse antes
del primer frame y se resuelven con prioridad `LAYER > FRAME > GLOBAL`.
La API `process_animation` devuelve `Animation`, una secuencia de
`AnimationFrame` con `number`, `duration_ms` e `image`. `process_text` y
`process_text_svg` rechazan documentos 1.2 para evitar seleccionar
silenciosamente un frame.

No está implementada la exportación animada SVG, ni las operaciones
derivadas de `GROUP`, scripts ni plugins. Las curvas Bézier quedan fuera de
PIXL 1.1/1.2. Las reglas geométricas normativas y sus casos de prueba están
en [`tests/corpus/`](./tests/corpus/).

Las versiones del motor se identificarán como `vPY`. La futura implementación
independiente en Rust se denominará PIXL MotorRU y usará `vRU`; no forma parte
de esta distribución.

## Plugin trust

`plugin_trust` está apartado temporalmente en `Mythree/Beta/plugin_trust` y
no forma parte de MotorPY ni de sus dependencias activas.
