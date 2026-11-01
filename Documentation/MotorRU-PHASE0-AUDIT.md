# PIXL MotorRU — Auditoría de Fase 0

Fecha de inspección: 2026-10-30

## Alcance y estado

Esta auditoría prepara la reimplementación nativa de PIXL MotorRU. No crea
MotorRU-Core, no integra Mythree y no modifica MotorPY. El único motor
existente en el entorno inspeccionado es MotorPY.

Toolchain Rust verificado:

- `rustc 1.99.1`
- `cargo 1.99.1`
- `stable-x86_64-pc-windows-gnu` (predeterminado)

No existe todavía `PIXL-MotorRU`, `PIXL-MotorRU-Core`, un workspace Cargo ni
una suite Rust.

## Estructura real

```text
C:\Comandos\Mythree\
├── Proyectos\
│   └── Motors\
│       ├── Code\
│       │   └── PIXL-MotorPY\
│       └── Documentation\
└── Beta\
    └── plugin_trust\
```

El corpus actual está en `PIXL-MotorPY/tests/corpus`. Es neutral en formato
(`.pixl` y `manifest.json`), pero todavía no está separado en
`Motors/Tests`; MotorRU deberá consumirlo sin copiarlo ni convertir snapshots
de Python en la autoridad normativa.

## Mapa de MotorPY

| Área | Ubicación | Responsabilidad |
|---|---|---|
| Lexer | `pixl/lexer.py` | Tokens, comentarios y ubicaciones |
| Parser | `pixl/parser.py` | Declaraciones, comandos y recuperación |
| Document/IR | `pixl/document.py` | Documento, capas, frames, grupos y comandos |
| Validator | `pixl/validator.py` | Reglas de versión, geometría, colores y ámbitos |
| Resolver | `pixl/resolver.py` | Colores literales, grupos y transparencia de desconocidos |
| Render model | `pixl/render_model.py` | Estados de píxel, rasterización y escena |
| Raster/composición | `pixl/renderer.py` | RGBA, capas y composición alfa |
| SVG | `pixl/renderers/svg_backend.py` | Serialización semántica de la escena |
| Pipeline | `pixl/pipeline.py` | APIs estáticas, animación y lectura de archivos |
| Diagnósticos | `pixl/diagnostics.py`, `pixl/errors.py` | Severidad, ubicación y registro extensible |
| GUI/CLI | `pixl/gui.py`, `main.py` | Interfaz tkinter y comandos de terminal |
| Pruebas | `tests/test_pixl.py` | Regresión funcional y corpus |

## Reglas normativas que MotorRU debe respetar

- PIXL 1.1 es estático; PIXL 1.2 requiere frames no vacíos.
- `TIPE(bit)` produce raster; `TIPE(vec)` produce SVG estático.
- Colores: `HEX8`, `RGBA(1)` y `RGBA(2)` deben converger a RGBA de 8 bits.
- Capas mantienen estados independientes: `UNDECLARED`, `DECLARED`,
  `INVISIBLE`.
- Comandos normales sobrescriben; `fill` solo escribe `UNDECLARED`.
- Capas se componen en orden de declaración mediante alfa.
- Grupos usan prioridad `LAYER > FRAME > GLOBAL`.
- Grupo desconocido: warning `PIXL040` y color transparente.
- Errores léxicos, sintácticos o semánticos bloquean el renderer.
- Polígonos repetidos producen `PIXL038`; polígonos colineales producen
  `PIXL039`.
- Figuras válidas fuera del canvas se recortan; `pixel`, `rect` y `fill`
  requieren rangos dentro del canvas.
- SVG animado no está soportado y produce `PIXL063`.
- Diagnósticos conservan archivo, línea y carácter.

La especificación normativa completa está en
[`SPECIFICATION.md`](../Code/PIXL-MotorPY/PIXL-MotorPY-v1-BETA/SPECIFICATION.md).

## Decisiones arquitectónicas para Rust

MotorRU será un crate independiente, sin Python ni dependencia de MotorPY.
La separación propuesta es:

1. `syntax`: lexer y parser con ubicaciones.
2. `model`: documento e IR validable.
3. `diagnostics`: severidad, códigos, ubicaciones y registro extensible.
4. `semantic`: validación y resolución de grupos.
5. `render_model`: escena y estados de píxel.
6. `raster`: rasterización y composición RGBA.
7. `svg`: serialización SVG estática.
8. `animation`: frames y duraciones.
9. `api`: fachada pública.
10. `cli` y `gui`: superficies de usuario separadas del núcleo.

El núcleo no debe importar GUI ni backend gráfico. La primera implementación
debe minimizar dependencias y justificar cualquier crate externo. La GUI no
se considerará validada por compilar: requerirá una sesión visual real.

## Diferencias y riesgos

1. El parser Python mantiene estados de contexto y permite que grupos se
   resuelvan después de su uso; Rust debe conservar esa semántica sin copiar
   la implementación.
2. El modelo Python representa la escena ya rasterizada también para SVG;
   MotorRU debe documentar si mantiene esa estrategia o usa una IR geométrica,
   pero la equivalencia debe verificarse semánticamente.
3. La composición alfa y el redondeo de `RGBA(1)` pueden producir diferencias
   de un canal; deben fijarse con pruebas RGBA exactas.
4. La validación y el renderer deben estar separados para garantizar que
   ningún documento inválido llegue a renderización.
5. El corpus actual necesita una ubicación compartida y metadatos suficientes
   para diagnósticos, raster, SVG y animaciones.
6. La GUI Rust añade riesgo de dependencias y disponibilidad del entorno; no
   debe contaminar el crate de procesamiento.

## Lagunas detectadas

- No hay aún proyecto Cargo ni implementación MotorRU.
- No hay pruebas de compatibilidad cruzada Python/Rust.
- El corpus no está ubicado en una carpeta común de Motors.
- No hay referencias RGBA completas para todos los casos raster.
- No hay manifest semántico detallado para todos los casos SVG.
- No existe auditoría visual reproducible dentro de Rust.
- Los artefactos `__pycache__` y `.pytest_cache` deben excluirse de cualquier
  migración o distribución.

## Plan de implementación posterior

1. Crear el crate independiente MotorRU y su contrato público.
2. Implementar lexer, parser, modelo, diagnósticos y validator.
3. Implementar resolver, escena, raster y composición.
4. Implementar SVG estático y animación 1.2.
5. Implementar API y CLI; probar ambos desde terminal.
6. Elegir e implementar GUI sin introducir MotorRU-Core.
7. Ejecutar el corpus neutral contra ambos motores.
8. Investigar toda diferencia y repetir pruebas completas.
9. Documentar compilación, API, limitaciones y auditoría.

**Veredicto de Fase 0: COMPLETADA.** La implementación de MotorRU puede
comenzar. MotorRU-Core e integración Mythree permanecen bloqueados hasta la
aprobación explícita de MotorRU.
