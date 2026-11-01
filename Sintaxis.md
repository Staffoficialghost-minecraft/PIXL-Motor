# Sintaxis de PIXL

PIXL es un lenguaje de descripción gráfica para crear imágenes rasterizadas,
salidas vectoriales y animaciones. Esta guía describe la sintaxis implementada
actualmente por MotorPY.

## Documento mínimo

Un documento estático PIXL 1.1 debe declarar la versión, el lienzo, el tipo de
salida, el sistema de color y al menos una capa:

```pixl
PIXL(1.1)
CANVAS(4, 3)
TIPE(bit)
HEX8()

LAYER(1)
pixel(1, 1, FF0000FF)
```

Las coordenadas comienzan en `1`. `(1, 1)` representa la esquina superior
izquierda.

## Comentarios y espacios

Los comentarios comienzan con `#` y terminan al final de la línea:

```pixl
# Comentario completo
PIXL(1.1) # Comentario al final de una instrucción
```

También se permiten líneas vacías, espacios y tabulaciones.

## Configuración

### Versión

```pixl
PIXL(1.1)
```

`PIXL(1.1)` se utiliza para documentos estáticos. `PIXL(1.2)` se utiliza para
animaciones y requiere uno o más `FRAME`.

### Lienzo

```pixl
CANVAS(ancho, alto)
```

El ancho y el alto deben ser enteros positivos:

```pixl
CANVAS(32, 32)
```

### Tipo de salida

Salida raster:

```pixl
TIPE(bit)
```

Salida vectorial SVG:

```pixl
TIPE(vec)
```

Una animación no puede exportarse a SVG en esta versión.

### Sistema de color

Debe declararse un único sistema de color:

```pixl
HEX8()
RGBA(1)
RGBA(2)
```

`HEX8()` utiliza `RRGGBBAA`, con canales hexadecimales entre `00` y `FF`:

```pixl
FF0000FF
```

`RGBA(2)` utiliza canales enteros entre `0` y `255`:

```pixl
255, 0, 0, 255
```

`RGBA(1)` utiliza canales decimales entre `0` y `1`:

```pixl
1, 0, 0, 1
```

El orden de los canales siempre es rojo, verde, azul y alfa.

## Capas

Una capa se declara con:

```pixl
LAYER(numero)
```

Los comandos gráficos deben aparecer después de una capa. Cada capa mantiene
sus propios estados de píxel y las capas se componen en el orden declarado:

```pixl
PIXL(1.1)
CANVAS(2, 1)
TIPE(bit)
HEX8()

LAYER(1)
pixel(1, 1, FF0000FF)

LAYER(2)
pixel(2, 1, 0000FFFF)
```

Los números de capa deben ser enteros positivos y no pueden repetirse dentro
del mismo documento.

## Grupos de color

Los grupos permiten reutilizar colores:

```pixl
GROUP = rojo(FF0000FF)

LAYER(1)
pixel(1, 1, rojo)
```

Los nombres de grupo se escriben en minúsculas. Pueden declararse en los
ámbitos global, de frame o de capa. La prioridad de resolución es:

```text
LAYER > FRAME > GLOBAL
```

Las declaraciones pueden aparecer antes o después de sus usos dentro del
ámbito válido. Un grupo duplicado produce `PIXL022`. Un grupo desconocido
produce la advertencia `PIXL040` y transparencia.

## Comandos gráficos

### `pixel`

Dibuja un píxel:

```pixl
pixel(x, y, color)
```

Ejemplo:

```pixl
pixel(2, 3, FF00FFFF)
```

### `inv`

Marca un píxel como invisible:

```pixl
inv(x, y)
```

Una instrucción gráfica posterior puede volver a declarar ese píxel.

### `rect`

Por defecto, dibuja un rectángulo relleno. Los rangos son inclusivos:

```pixl
rect(xInicio-xFinal, yInicio-yFinal, color)
```

Ejemplo:

```pixl
rect(1-4, 1-3, 00FF00FF)
```

Para un contorno:

```pixl
rect(1-4, 1-3, 00FF00FF, outline, 1, c)
```

Las posiciones de contorno son:

- `i`: interior.
- `c`: centrada.
- `e`: exterior.

El grosor predeterminado es `1` y la posición predeterminada es `c`.

### `fill`

Escribe únicamente sobre píxeles `UNDECLARED` dentro del rango:

```pixl
fill(1-4, 1-3, 000000FF)
```

`fill` no es un flood fill. No busca regiones conectadas y no sobrescribe
píxeles declarados o invisibles.

### `circle`

```pixl
circle(x, y, radio, color, fill)
circle(x, y, radio, color, outline, grosor, posicion)
```

El radio debe ser un entero positivo:

```pixl
circle(5, 5, 3, FF0000FF, fill)
```

### `ellipse`

```pixl
ellipse(x1, y1, x2, y2, color, fill)
ellipse(x1, y1, x2, y2, color, outline, grosor, posicion)
```

Los límites deben ser estrictamente crecientes:

```pixl
ellipse(2, 3, 8, 6, 0000FFFF, fill)
```

### `polygon`

Un polígono requiere al menos tres vértices:

```pixl
polygon(x1, y1, x2, y2, x3, y3, color, fill)
```

Ejemplo:

```pixl
polygon(2, 2, 8, 2, 5, 7, FF00FFFF, fill)
```

También admite contorno:

```pixl
polygon(2, 2, 8, 2, 5, 7, FF00FFFF, outline, 2, c)
```

No se permiten vértices repetidos (`PIXL038`) ni polígonos de área cero
(`PIXL039`).

## Estados y sobrescritura

Cada píxel de cada capa puede estar en uno de estos estados:

- `UNDECLARED`: todavía no fue declarado.
- `DECLARED`: contiene un comando gráfico.
- `INVISIBLE`: fue marcado con `inv`.

Las instrucciones gráficas normales sobrescriben las anteriores:

```pixl
pixel(1, 1, FF0000FF)
pixel(1, 1, 0000FFFF)
```

El resultado final es azul. `fill` es la excepción y solo escribe sobre
posiciones `UNDECLARED`.

## Límites y recorte

`pixel`, `rect` y `fill` deben permanecer dentro del canvas. Las coordenadas
negativas, límites invertidos y posiciones fuera del canvas son inválidos.

Las figuras `circle`, `ellipse` y `polygon` pueden cruzar los límites del
canvas; la parte visible se recorta. Una figura completamente fuera no cambia
ningún píxel si sus argumentos geométricos son válidos.

## Animaciones PIXL 1.2

Un documento 1.2 requiere frames no vacíos:

```pixl
PIXL(1.2)
CANVAS(4, 2)
TIPE(bit)
HEX8()

FRAME(1, 100)
    LAYER(1)
        pixel(1, 1, FF0000FF)

FRAME(2, 100)
    LAYER(1)
        pixel(2, 1, 0000FFFF)
```

El número de frame debe ser positivo y estrictamente creciente. La duración
debe ser un entero positivo expresado en milisegundos.

Cada frame es independiente y no hereda contenido de otro frame.

## Uso desde Python

Procesar texto estático:

```python
from pixl import process_text

image, diagnostics = process_text(source)
if diagnostics.has_errors:
    for diagnostic in diagnostics.items:
        print(diagnostic.code, diagnostic.description)
```

Procesar un archivo:

```python
from pixl import process_file

image, diagnostics = process_file("dibujo.pixl")
```

Procesar una animación:

```python
from pixl import process_animation

animation, diagnostics = process_animation(source)
if animation is not None:
    for frame in animation:
        print(frame.number, frame.duration_ms)
```

Para obtener SVG desde texto estático:

```python
from pixl import process_text_svg

svg, diagnostics = process_text_svg(source)
```

## Errores y validación

Los documentos con errores léxicos, sintácticos o de validación no llegan al
renderer. Los diagnósticos incluyen código, archivo, línea y carácter cuando
corresponde.

Ejemplos inválidos:

```pixl
# Coordenada fuera del lienzo
pixel(0, 1, FF0000FF)

# Comando fuera de una capa
pixel(1, 1, FF0000FF)

# FRAME en un documento 1.1
FRAME(1, 100)
```

## Fuera del alcance actual

Esta versión no incluye:

- SVG animado.
- Curvas Bézier.
- Operaciones matemáticas derivadas de `GROUP`.
- Scripts.
- Plugins.
- Funcionalidades no definidas por la especificación PIXL.
