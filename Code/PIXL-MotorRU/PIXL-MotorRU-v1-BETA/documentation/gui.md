# GUI de MotorRU

La feature `gui` utiliza `eframe` para proporcionar una interfaz interactiva
similar a MotorPY:

- Logo PIXL integrado en la barra superior.
- Icono PIXL integrado en la ventana, el ejecutable y la barra de tareas de
  Windows.
- **Abrir** archivos `.pixl`.
- **Guardar** el texto editado.
- **Procesar** el documento actual.
- Editar el código PIXL en el panel izquierdo.
- Desplazarse vertical y horizontalmente por documentos largos.
- Ver el raster y los diagnósticos en la misma ventana.
- Ver transparencia con un fondo ajedrezado: gris claro/gris oscuro para
  píxeles transparentes y mezcla proporcional para alfa parcial.

```powershell
cargo run -- --gui ruta\archivo.pixl
```

También se puede abrir directamente con doble clic sobre el ejecutable. Sin
argumentos se inicia un documento vacío listo para editar:

```powershell
PIXL MotorRU-Latest.exe
```

También puede iniciarse desde el binario compilado:

```powershell
target\debug\pixl-motorru.exe --gui ruta\archivo.pixl
```

La GUI muestra directamente raster. Para `TIPE(vec)` se debe usar la CLI para
exportar SVG; las animaciones se procesan mediante la API/CLI y se exportan
como frames PPM.

La validación visual requiere una sesión gráfica interactiva de Windows.

El ejecutable release se distribuye localmente como
`..\..\..\Exe\PIXL MotorRU-Latest.exe`.

En el entorno actual está seleccionado `stable-x86_64-pc-windows-gnu`.
La comprobación de código pasa, pero el enlace del ejecutable puede requerir
un toolchain Windows GNU funcional: si `dlltool` falla al crear la import
library de `kernel32.dll`, hay que reparar MinGW/GNU o instalar y seleccionar
un target MSVC antes de ejecutar la GUI.
