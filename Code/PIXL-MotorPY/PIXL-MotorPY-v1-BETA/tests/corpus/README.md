# Corpus común PIXL

Este corpus contiene entradas y expectativas independientes de la
implementación. MotorPY y MotorRU deben consumir los mismos archivos.

Cada caso incluye:

- un archivo `.pixl`;
- una entrada en `manifest.json`;
- resultado esperado o diagnóstico esperado.

Las referencias raster se expresan como píxeles RGBA en coordenadas 1-based.
Las referencias SVG se verifican semánticamente: dimensiones, elementos,
colores y transparencia, no igualdad textual.
