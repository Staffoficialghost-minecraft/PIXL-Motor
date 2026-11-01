# Especificación común de PIXL 1.1 y 1.2

Este documento describe las reglas implementadas por PIXL MotorPY (`vPY`).
PIXL MotorRU (`vRU`) deberá implementar estas mismas reglas en una etapa
posterior. Las versiones del motor no son versiones del lenguaje.

## Versiones

- `PIXL(1.1)` es estático y no permite `FRAME`.
- `PIXL(1.2)` requiere uno o más frames no vacíos.
- `FRAME(numero, duracion_ms)` requiere un número entero positivo,
  estrictamente creciente, y una duración entera positiva en milisegundos.
- Cada frame es independiente y no hereda contenido de otro frame.

## Configuración

`CANVAS(ancho, alto)` usa dimensiones enteras positivas y coordenadas
basadas en 1. `TIPE(bit)` genera raster y `TIPE(vec)` genera SVG. Los sistemas
de color son `HEX8()`, `RGBA(1)` y `RGBA(2)`.

## Capas y grupos

Las capas tienen números enteros positivos y no pueden repetirse dentro del
mismo documento o frame. Las capas se componen en el orden declarado; la
especificación no asigna todavía semántica adicional al número de capa.

Los grupos pueden ser globales, de frame o de capa. Las referencias se
resuelven con prioridad `LAYER > FRAME > GLOBAL`. Un grupo inexistente genera
`PIXL040` y transparencia. Un color inválido en una declaración `GROUP` es
un error bloqueante.

## Estados y comandos

Cada capa mantiene estados independientes `UNDECLARED`, `DECLARED` e
`INVISIBLE`. `pixel`, `rect`, `circle`, `ellipse` y `polygon` sobrescriben
el contenido anterior. `inv` establece invisibilidad. `fill` solo escribe
posiciones `UNDECLARED`; no es flood fill.

`rect` mantiene el modo relleno legacy. Su modo `outline`, y los modos
`outline` de las figuras, aceptan grosor positivo y posición `i` (interior),
`c` (centrada) o `e` (exterior). El grosor predeterminado es 1 y la posición
predeterminada es `c`.

El relleno de figuras se evalúa en centros de píxel `(x - 0.5, y - 0.5)`,
con bordes incluidos. Los contornos se evalúan como bandas de distancia al
borde ideal. Las figuras se recortan al canvas durante el rasterizado.

Las reglas geométricas son:

- `circle` requiere radio entero positivo.
- `ellipse` requiere límites estrictamente crecientes.
- `polygon` requiere al menos tres vértices enteros, sin vértices repetidos
  y con área distinta de cero. Los vértices repetidos producen `PIXL038` y
  los vértices colineales producen `PIXL039`.
- Una figura puede estar parcial o completamente fuera del canvas. Se acepta
  si sus argumentos son válidos y se recorta; si no intersecta, no cambia
  ningún píxel y no genera un error adicional.
- Los rangos de `rect` y `fill` son inclusivos y deben quedar dentro del
  canvas. Coordenadas negativas o límites invertidos son inválidos.

La composición de capas se realiza en orden de declaración con composición
alfa sobre RGBA. Los píxeles transparentes no borran una capa inferior.

## APIs

`process_text` procesa documentos estáticos. `process_text_svg` procesa
documentos estáticos y no acepta 1.2. `process_animation` y
`process_animation_file` procesan 1.2 y devuelven `Animation`, una secuencia
de `AnimationFrame(number, duration_ms, image)`. `process_file` procesa
documentos estáticos desde disco. En la GUI, `TIPE(bit)` abre el raster
ampliado en una ventana separada y `TIPE(vec)` abre el SVG ampliado en el
navegador predeterminado.

`TIPE(vec)` en una animación produce el error bloqueante `PIXL063`: esta
versión no exporta SVG animado.

## Fuera de alcance

No forman parte de esta versión la exportación SVG animada,
curvas Bézier, operaciones derivadas de `GROUP`, scripts, plugins,
instaladores ni MotorRU.

## Casos inválidos

Un documento con errores léxicos, sintácticos o de validación no llega al
renderer y devuelve `image is None`. Los diagnósticos conservan archivo,
línea y carácter. Los casos normativos ejecutables están en
[`tests/corpus/`](./tests/corpus/).
