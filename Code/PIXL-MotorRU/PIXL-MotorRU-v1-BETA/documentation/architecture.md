# Arquitectura actual

MotorRU se divide en una biblioteca (`src/lib.rs`), una CLI (`src/main.rs`) y
una GUI opcional (`src/gui.rs`).
La biblioteca no importa Python ni MotorPY.

## Flujo

1. `parse` transforma el texto PIXL en un documento interno.
2. `validate` bloquea versiones, dimensiones, coordenadas, frames y geometría
   inválidas.
3. `render` construye imágenes RGBA independientes por capa y compone alfa.
4. `to_svg` serializa la imagen resultante para `TIPE(vec)`.
5. `process_animation` procesa cada frame de forma independiente.

El corpus de integración se consume desde
`../../PIXL-MotorPY/PIXL-MotorPY-v1-BETA/tests/corpus` mientras se define una ubicación neutral
compartida para ambos motores. No se usa ningún snapshot de Python como
autoridad normativa.

## Estado de superficies

- Biblioteca: implementada inicialmente para procesamiento estático y frames.
- Raster: píxeles, rectángulos, fill, círculos, elipses, polígonos, capas y
  composición alfa.
- SVG: salida estática semántica basada en RGBA.
- CLI: ayuda, archivos estáticos, exportación PPM/SVG y animaciones por frame.
- GUI: compilable y funcional en código, pendiente de validación visual manual.
- MotorRU-Core: no iniciado.
- Mythree: no integrado.
