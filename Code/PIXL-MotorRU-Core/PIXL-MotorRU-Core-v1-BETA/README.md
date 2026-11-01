# PIXL MotorRU-Core

Core BETA independiente para analizar, validar y normalizar PIXL. El crate
`pixl_motorru_core` no contiene GUI, CLI, rasterización, exportadores ni
dependencias gráficas.

```rust
use pixl_motorru_core::process_text;

let (document, diagnostics) = process_text(source, "drawing.pixl");
assert!(!diagnostics.has_errors());
```

La licencia declarada es `Apache-2.0`. El texto íntegro oficial deberá
incorporarse desde la fuente del proyecto cuando `Motors` se publique.
