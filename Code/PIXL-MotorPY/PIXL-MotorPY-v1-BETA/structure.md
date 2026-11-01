# Cómo escribir un archivo PIXL

Esta guía explica la estructura de los documentos compatibles con PIXL
MotorPY (`vPY`).

## 1. Estructura básica

Un documento estático PIXL 1.1 debe declarar:

1. La versión del lenguaje.
2. El tamaño del lienzo.
3. El tipo de representación.
4. El sistema de color.
5. Una o más capas.
6. Los comandos gráficos de cada capa.

Ejemplo mínimo:

```pixl
PIXL(1.1)
CANVAS(4, 3)
TIPE(bit)
HEX8()

LAYER(1)
pixel(1, 1, FF0000FF)
```

Las coordenadas empiezan en `1`. En el ejemplo, el píxel `(1, 1)` es la
esquina superior izquierda.

## 2. Comentarios y formato

Los comentarios comienzan con `#` y terminan al final de la línea:

```pixl
# Este es un comentario
PIXL(1.1) # También puede aparecer después de una instrucción
```

Se permiten líneas vacías y espacios o tabulaciones entre elementos.

## 3. Configuración

### Versión

```pixl
PIXL(1.1)
```

`PIXL(1.1)` crea un documento estático. `PIXL(1.2)` se usa para
animaciones y requiere al menos un `FRAME`.

### Lienzo

```pixl
CANVAS(ancho, alto)
```

Ambos valores deben ser enteros positivos:

```pixl
CANVAS(32, 32)
```

### Tipo

El modo raster se declara así:

```pixl
TIPE(bit)
```

El modo vectorial genera SVG automáticamente:

```pixl
TIPE(vec)
```

### Sistema de color

Debe elegirse un único sistema de color:

```pixl
HEX8()
RGBA(1)
RGBA(2)
```

`HEX8()` usa `RRGGBBAA` con canales de `00` a `FF`:

```pixl
FF0000FF
```

`RGBA(2)` usa canales de `0` a `255`:

```pixl
255, 0, 0, 255
```

`RGBA(1)` usa canales de `0` a `1`:

```pixl
1, 0, 0, 1
```

El orden siempre es rojo, verde, azul y alfa.

## 4. Capas

Una capa se selecciona con:

```pixl
LAYER(numero)
```

Los comandos gráficos deben aparecer después de una declaración `LAYER`.
Las capas mantienen estados de píxel independientes y se componen en el orden
en que aparecen. Una capa posterior puede cubrir una capa anterior si contiene
píxeles opacos.

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

## 5. Grupos de color

Los grupos permiten reutilizar colores:

```pixl
GROUP = rojo(FF0000FF)

LAYER(1)
pixel(1, 1, rojo)
```

Los nombres deben escribirse en minúsculas. Pueden declararse en estos
ámbitos:

- global, antes de las capas;
- dentro de una capa;
- dentro de un frame en PIXL 1.2.

La prioridad de resolución es:

```text
LAYER > FRAME > GLOBAL
```

Un grupo inexistente genera una advertencia `PIXL040` y produce transparencia.
Las declaraciones pueden aparecer antes o después de sus usos dentro del
ámbito válido. Una declaración duplicada produce `PIXL022`.

## 6. Comandos gráficos

### `pixel`

Dibuja un píxel:

```pixl
pixel(x, y, color)
```

Ejemplos:

```pixl
pixel(2, 3, FF00FFFF)
```

Con `RGBA(2)`:

```pixl
pixel(2, 3, 255, 0, 255, 255)
```

### `inv`

Declara un píxel como invisible:

```pixl
inv(x, y)
```

Ejemplo:

```pixl
inv(2, 2)
```

Una instrucción posterior como `pixel` puede volver a declarar ese píxel.

### `rect`

Por defecto, `rect` dibuja un rectángulo relleno. Los rangos incluyen ambos
límites:

```pixl
rect(xInicio-xFinal, yInicio-yFinal, color)
```

Ejemplo:

```pixl
rect(1-4, 1-3, 00FF00FF)
```

También admite contorno:

```pixl
rect(1-4, 1-3, 00FF00FF, outline, 1, c)
```

Las posiciones de contorno son:

- `i`: interior;
- `c`: centrada;
- `e`: exterior.

El grosor predeterminado es `1` y la posición predeterminada es `c`.

### `fill`

`fill` escribe únicamente sobre posiciones `UNDECLARED` dentro del rango:

```pixl
fill(1-4, 1-3, 000000FF)
```

No es un flood fill. No busca regiones conectadas y no sobrescribe píxeles
declarados o invisibles.

### Geometría y límites

Las figuras pueden cruzar el borde del canvas: la parte visible se recorta.
Una figura completamente fuera no dibuja píxeles, pero sigue siendo válida.
En cambio, `pixel`, `rect` y `fill` requieren coordenadas dentro del canvas.

Un polígono debe tener al menos tres vértices enteros, sin repetir ninguno y
sin que todos sean colineales:

```pixl
# Inválido: PIXL038, vértice repetido
polygon(1, 1, 2, 2, 1, 1, FFFFFFFF, fill)

# Inválido: PIXL039, área cero
polygon(1, 1, 2, 2, 3, 3, FFFFFFFF, fill)
```

### `circle`

```pixl
circle(x, y, radio, color, fill)
circle(x, y, radio, color, outline, grosor, posicion)
```

Ejemplos:

```pixl
circle(5, 5, 3, FF0000FF, fill)
circle(5, 5, 3, FF0000FF, outline, 1, c)
```

### `ellipse`

```pixl
ellipse(x1, y1, x2, y2, color, fill)
ellipse(x1, y1, x2, y2, color, outline, grosor, posicion)
```

Los cuatro primeros valores forman el rectángulo delimitador:

```pixl
ellipse(2, 3, 8, 6, 0000FFFF, fill)
```

### `polygon`

Un polígono necesita al menos tres vértices. Cada vértice se expresa como un
par `x, y`:

```pixl
polygon(x1, y1, x2, y2, x3, y3, color, fill)
```

Ejemplo:

```pixl
polygon(2, 2, 8, 2, 5, 7, FF00FFFF, fill)
```

También puede utilizar contorno:

```pixl
polygon(2, 2, 8, 2, 5, 7, FF00FFFF, outline, 2, c)
```

## 7. Sobrescritura y estados

Cada píxel de cada capa puede estar en uno de estos estados:

- `UNDECLARED`: todavía no fue declarado;
- `DECLARED`: contiene un comando gráfico;
- `INVISIBLE`: fue declarado como invisible.

Las instrucciones gráficas normales sobrescriben las anteriores:

```pixl
pixel(1, 1, FF0000FF)
pixel(1, 1, 0000FFFF)
```

El resultado final es azul.

`fill` es la excepción: solo escribe sobre posiciones `UNDECLARED`.

## 8. Animaciones PIXL 1.2

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

Las duraciones son enteros positivos en milisegundos. Los números de frame
deben ser positivos y aumentar estrictamente.

Cada frame es independiente: el contenido del frame anterior no se hereda.
Para procesar una animación se utiliza `process_animation()` o:

```powershell
python main.py --animation animacion.pixl
```

`PIXL(1.2)` no debe procesarse con `process_text()` ni con
`process_text_svg()`.

## 9. Errores habituales

Estas entradas son inválidas:

```pixl
# Coordenada fuera del lienzo
pixel(0, 1, FF0000FF)

# Comando fuera de una capa
pixel(1, 1, FF0000FF)

# FRAME en un documento 1.1
FRAME(1, 100)
```

Los diagnósticos incluyen código, archivo, línea, carácter, categoría y
descripción cuando existe una definición registrada.

## 10. Procesamiento desde Python

```python
from pixl import process_text, process_text_svg

image, diagnostics = process_text(source)
if diagnostics.has_errors:
    for diagnostic in diagnostics.items:
        print(diagnostic.code, diagnostic.description)
else:
    print(image.width, image.height)
```

Para archivos:

```python
from pixl import process_file

image, diagnostics = process_file("dibujo.pixl")
```

Para animaciones:

```python
from pixl import process_animation

animation, diagnostics = process_animation(source)
if animation is not None:
    for frame in animation:
        print(frame.number, frame.duration_ms, frame.image)
```

## 11. Funciones fuera del alcance actual

No deben utilizarse como parte de esta versión:

- exportación SVG animada;
- curvas Bézier;
- operaciones matemáticas derivadas de `GROUP`;
- scripts;
- plugins;
- MotorRU.
