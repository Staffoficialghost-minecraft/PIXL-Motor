# PIXL Motor Installer

Instalador de terminal para las distribuciones publicadas de PIXL Motor.

## Compilación

```powershell
cd Installer
cargo fmt --check
cargo check
cargo test
cargo build --release
```

El ejecutable se genera en `target/release/pixl-motor-installer.exe`.

## Modalidades

El instalador consulta Releases de GitHub y usa Tags como fallback. Solo
muestra una modalidad cuando existen assets identificables:

- MotorPY mediante pip: requiere los paquetes `pixl_motorpy` y
  `pixl_motorpy_core` en la Release.
- MotorPY como proyecto: descarga el tarball de la fuente.
- MotorRU-Core: prepara la fuente del crate para una dependencia Cargo local.
- MotorRU: instala un `.exe` cuyo nombre corresponde a MotorRU.

Los hashes SHA-256 publicados por GitHub se verifican cuando están presentes.
Los archivos descargados nunca se ejecutan automáticamente. Las instalaciones
existentes no se sobrescriben.
