# Contribuir a PIXL Motor

Gracias por tu interés en PIXL Motor. Las contribuciones son bienvenidas y
ayudan a hacer que el proyecto sea más estable, mantenible y útil en
diferentes plataformas.

## Formas de contribuir

Puedes contribuir de muchas maneras:

- Desarrollando código.
- Corrigiendo errores.
- Reportando bugs.
- Mejorando la documentación.
- Añadiendo o mejorando pruebas.
- Revisando código.
- Compilando y probando en diferentes plataformas.
- Ayudando con la compatibilidad multiplataforma, especialmente en Linux.
- Proponiendo y discutiendo mejoras técnicas.

## Soporte de plataformas

Actualmente PIXL Motor necesita ayuda para:

- Compilar y probar PIXL Motor en Linux.
- Identificar problemas específicos de Linux.
- Mejorar la compatibilidad multiplataforma.

El soporte de Linux todavía está siendo investigado y mejorado; no debe
considerarse oficialmente soportado hasta que haya sido confirmado.

Cualquier persona con experiencia en Rust, Python, Linux, compilación de
software o empaquetado puede ayudar en estas áreas.

## Antes de comenzar

Antes de comenzar un trabajo importante:

1. Revisa los issues existentes.
2. Evita duplicar trabajo que ya esté en progreso.
3. Abre una discusión o un issue antes de realizar cambios grandes en la
   arquitectura o en la sintaxis.
4. No introduzcas funcionalidades especulativas sin discutirlas previamente.

Estas prácticas son especialmente importantes porque PIXL tiene una
especificación del lenguaje y una arquitectura que deben mantenerse coherentes
entre sus implementaciones y componentes.

## Directrices de desarrollo

Sigue estas directrices generales:

- Mantén el código modular.
- Separa claramente las responsabilidades.
- No mezcles la lógica del lenguaje con el comportamiento específico de los
  backends.
- Evita dependencias innecesarias.
- Conserva la compatibilidad con la arquitectura existente.
- Escribe código claro y mantenible.
- Añade o actualiza pruebas cuando corresponda.
- Evita incluir cambios grandes no relacionados en el mismo pull request.

### Rust

Para cambios en Rust, ejecuta los comandos relevantes documentados por el
componente que estés modificando. En general, se recomienda ejecutar:

```text
cargo fmt
cargo check
cargo test
```

### Python

Para cambios en Python:

- Ejecuta las pruebas disponibles para el componente.
- Mantén las dependencias al mínimo.
- Sigue las instrucciones de desarrollo existentes del componente.

No asumas que todos los componentes utilizan los mismos comandos o la misma
estructura de pruebas. Evita inventar comandos que no existan o no sean
aplicables en el repositorio.

## Cambios en el lenguaje PIXL

PIXL es un lenguaje en desarrollo. Los cambios en el lenguaje requieren
especial cuidado:

- No añadas comandos ni funcionalidades del lenguaje sin discutirlo
  previamente.
- Los cambios de sintaxis requieren revisión antes de implementarse.
- La especificación del lenguaje tiene prioridad sobre los comportamientos
  improvisados de una implementación individual.
- Las nuevas funcionalidades deben considerar la compatibilidad futura y la
  arquitectura existente.

Cuando una funcionalidad sea compartida, los cambios deben preservar la
coherencia semántica entre MotorPY, MotorPY-Core, MotorRU y MotorRU-Core.

## Pull Requests

El proceso habitual de contribución es:

1. Crea un fork.
2. Crea una rama descriptiva.
3. Realiza los cambios.
4. Ejecuta las pruebas relevantes.
5. Abre un pull request explicando claramente qué cambió y por qué.

Los pull requests deben:

- Ser específicos y estar enfocados.
- Evitar archivos generados que no sean necesarios.
- Evitar cambios no relacionados.
- Incluir pruebas cuando corresponda.
- Explicar cualquier limitación específica de plataforma o verificación
  pendiente.

## Issues y reportes de bugs

Cuando sea posible, los reportes de bugs deben incluir:

- Sistema operativo.
- Versión del componente.
- Versión de Python o Rust.
- Pasos para reproducir el problema.
- Resultado esperado.
- Resultado obtenido.
- Mensajes de error o registros relevantes.

Incluye el ejemplo reproducible más pequeño disponible e indica si el
problema afecta al código fuente, al empaquetado, a la instalación o al
comportamiento durante la ejecución.

## Comunicación

Para comentarios generales, contribuciones, colaboración o ayuda, puedes
contactar con:

[**staffoficialghost@gmail.com**](mailto:staffoficialghost@gmail.com)

## Código de conducta

Mantén las conversaciones respetuosas y constructivas:

- Trata a los demás con respeto.
- Mantén constructivas las discusiones técnicas.
- No realices ataques personales.
- No discrimines.
- Enfoca las críticas en el código, los diseños y las ideas, no en las
  personas.

## Nota final

Gracias a todas las personas que ayudan a hacer PIXL Motor más estable,
mantenible y multiplataforma.
