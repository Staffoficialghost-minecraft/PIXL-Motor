# PIXL Motor

PIXL Motor es un proyecto de motor y lenguaje gráfico en desarrollo. El
repositorio contiene varias implementaciones, cores independientes,
herramientas de instalación, artefactos de distribución y documentación.

## Componentes

### MotorPY

Implementación de PIXL en Python. Incluye procesamiento del lenguaje,
validación, resolución y backends de renderizado.

Ruta:

`Code/PIXL-MotorPY/PIXL-MotorPY-v1-BETA`

### MotorPY-Core

Core independiente de MotorPY para analizar, validar, resolver y normalizar
documentos PIXL mediante una representación intermedia pública.

Ruta:

`Code/PIXL-MotorPY-Core/PIXL-MotorPY-Core-v1-BETA`

### MotorRU

Implementación nativa de PIXL en Rust.

Ruta:

`Code/PIXL-MotorRU/PIXL-MotorRU-v1-BETA`

### MotorRU-Core

Core independiente en Rust, sin GUI, CLI ni dependencias gráficas.

Ruta:

`Code/PIXL-MotorRU-Core/PIXL-MotorRU-Core-v1-BETA`

### PIXL Installer

Instalador de terminal para Windows escrito en Rust. Obtiene las generaciones
publicadas desde el manifiesto de distribución, fija la instalación a una
revisión concreta de la rama `main` y evita mezclar componentes de revisiones
distintas.

Ruta:

`Installer/`

## Distribución mediante manifiesto

Las generaciones PIXL se declaran en:

`distribution/manifest.json`

El manifiesto identifica:

- Generaciones disponibles.
- Estado de publicación.
- Componentes incluidos.
- Versiones internas.
- Rutas de distribución.
- Formatos.
- Dependencias entre componentes.
- Plataformas compatibles cuando corresponde.

Una generación incompleta no debe considerarse instalable. Actualmente se
declaran los componentes MotorPY, MotorPY-Core, MotorRU y MotorRU-Core.

## Estructura principal

```text
Code/
  PIXL-MotorPY/
  PIXL-MotorPY-Core/
  PIXL-MotorRU/
  PIXL-MotorRU-Core/
distribution/
  manifest.json
Installer/
  Cargo.toml
  Cargo.lock
  src/
Exe-librerias/
Documentation/
Examples/
```

## Requisitos

Los requisitos dependen del componente:

- Python 3.10 o posterior para MotorPY y MotorPY-Core.
- Rust y Cargo para MotorRU, MotorRU-Core y PIXL Installer.
- Pillow solo cuando se utiliza el backend de MotorPY que lo requiere.

La compatibilidad con Linux está en desarrollo y todavía requiere compilación,
pruebas y verificación adicionales. No se debe considerar soporte oficial
hasta que haya sido confirmado.

## MotorPY

Desde el directorio de MotorPY:

```bash
python -m pip install .
python -m pytest tests -q
```

MotorPY depende de MotorPY-Core. Para utilizar solamente el core, consulta su
propia documentación y configuración de empaquetado.

## MotorRU-Core

Desde el directorio de MotorRU-Core:

```bash
cargo fmt --check
cargo check
cargo test
```

MotorRU-Core es una biblioteca Rust independiente y debe incorporarse como
crate fuente mediante Cargo. Un archivo `.rlib` aislado no sustituye al crate
completo.

## PIXL Installer

Desde `Installer/`:

```powershell
cargo fmt --check
cargo check
cargo test
cargo build --release
```

El ejecutable release se genera en:

```text
Installer/target/release/pixl-motor-installer.exe
```

El instalador compilado para distribución se conserva en:

```text
Exe-librerias/PIXL-Installer/pixl-motor-installer.exe
```

El instalador no utiliza GitHub Releases ni Git Tags como catálogo de
versiones. Consulta la rama `main`, obtiene su SHA, descarga el manifiesto de
esa revisión y utiliza esa misma revisión para los archivos de la instalación.

## Pruebas y calidad

Antes de enviar cambios:

- Ejecuta las pruebas relevantes del componente modificado.
- Ejecuta `cargo fmt` y `cargo check` para cambios Rust.
- Mantén las dependencias mínimas.
- No mezcles la lógica del lenguaje con backends específicos.
- No incluyas archivos generados innecesarios.
- Añade o actualiza pruebas cuando el comportamiento cambie.

Consulta [CONTRIBUTING.md](CONTRIBUTING.md) para conocer las directrices de
contribución, reportes de bugs y pull requests.

## Estado del proyecto

PIXL Motor es software en desarrollo. Las implementaciones pueden tener
distintos niveles de madurez y verificación. No se deben asumir capacidades,
formatos o plataformas que no estén documentados y probados por el componente
correspondiente.

## Comunicación

Para comentarios generales, colaboración o ayuda:

[**staffoficialghost@gmail.com**](mailto:staffoficialghost@gmail.com)

## Licencia

Los componentes declaran Apache License 2.0 en sus metadatos cuando
corresponde. El texto íntegro oficial de la licencia deberá incorporarse desde
la fuente del proyecto al preparar la publicación final del repositorio.
