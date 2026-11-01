# PIXL Motor Installer

Instalador de terminal para las distribuciones declaradas en
[`distribution/manifest.json`](../distribution/manifest.json).

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

El instalador consulta la rama `main`, fija la instalación al SHA actual y
descarga el manifiesto y los archivos desde esa misma revisión. No utiliza
Releases ni Tags como catálogo de versiones. Solo muestra generaciones
completas y modalidades declaradas por el manifiesto:

- MotorPY mediante pip: instala primero MotorPY-Core y después MotorPY desde
  sus árboles fuente declarados.
- MotorPY como proyecto: extrae únicamente la ruta fuente declarada.
- MotorRU-Core: extrae únicamente el árbol fuente del crate.
- MotorRU: descarga el ejecutable declarado por el manifiesto.

Los hashes SHA-256 declarados por el manifiesto se verifican cuando están
presentes.
Los archivos descargados nunca se ejecutan automáticamente. Las instalaciones
existentes no se sobrescriben.
