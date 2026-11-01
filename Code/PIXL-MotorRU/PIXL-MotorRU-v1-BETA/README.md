# PIXL MotorRU

Implementación nativa e independiente de PIXL en Rust (`vRU`). No ejecuta
Python, no depende de MotorPY y no contiene MotorRU-Core.

El diseño y las decisiones de compatibilidad se documentan en
[`MotorRU-PHASE0-AUDIT.md`](../../../Documentation/MotorRU-PHASE0-AUDIT.md).

## Estado

La validación no visual cubre el núcleo, rasterizador, SVG estático,
animaciones, API pública y CLI. El motor no está aprobado como equivalente
completo a MotorPY; la GUI requiere validación visual manual.

```powershell
cargo test
cargo run -- --help
cargo run -- tests\example.pixl
cargo run -- tests\example.pixl --output resultado.ppm
cargo run -- tests\vector.pixl --output resultado.svg
cargo run -- --animation tests\animation.pixl --output-dir frames
cargo run -- --gui tests\example.pixl
```

El ejecutable release se encuentra en
`..\..\..\Exe\PIXL MotorRU-Latest.exe`. Al abrirlo con doble clic inicia la GUI con
un documento vacío; también puede recibir `--gui archivo.pixl`.

La exportación raster usa PPM P6 sin interpolación. La exportación de
animaciones genera un PPM por frame, conservando el número y la duración en
el nombre del archivo.

La GUI permite abrir, editar, guardar y procesar archivos PIXL, mostrando el
resultado raster y los diagnósticos en paneles separados. Su validación visual
requiere una sesión gráfica de Windows.

La arquitectura se describe en
[`documentation/architecture.md`](documentation/architecture.md). La
auditoría de preparación está en
[`MotorRU-PHASE0-AUDIT.md`](../../../Documentation/MotorRU-PHASE0-AUDIT.md).

La GUI raster usa la feature `gui` y amplía los píxeles sin interpolación.
Su validación visual requiere ejecutar el comando en una sesión gráfica.

La API está documentada en [`documentation/api.md`](documentation/api.md) y
la GUI en [`documentation/gui.md`](documentation/gui.md).
