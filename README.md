# Normalizador NQ4

Aplicación de escritorio portable para Windows, escrita en Rust, orientada a inspeccionar, crear, editar y normalizar etiquetas ID3 de archivos MP3 sin recodificar el audio.

## Interfaz de escritorio

La aplicación ya está organizada como un editor de escritorio completo:

- barra de menús: **Archivo, Editar, Herramientas, Ver y Ayuda**;
- barra de herramientas;
- panel de archivos con filtro/buscador;
- editor central de tags;
- panel configurable del perfil de normalización;
- barra de estado;
- ventanas separadas de **Preferencias, Diagnóstico ID3, Vista previa de portada y Acerca de**;
- confirmación antes de normalización por lote;
- progreso del lote sin bloquear toda la interfaz entre archivos;
- atajos de teclado;
- drag & drop;
- cambios pendientes marcados visualmente;
- guardar un archivo o todos;
- descartar cambios recargando el archivo;
- quitar archivos de la lista sin borrarlos del disco.

## Edición ID3

Permite crear o modificar:

- título;
- artista;
- álbum;
- género;
- año;
- número de pista;
- número de disco;
- comentario;
- portada JPEG/PNG.

## Perfil iPod seguro

La normalización:

- reescribe como **ID3v2.3**;
- elimina **TLEN**, evitando duraciones heredadas incorrectas;
- puede eliminar ID3v1 y APEv2 residuales;
- conserva los campos musicales comunes;
- puede convertir la portada a JPEG y limitarla a 600×600 (configurable);
- crea backup antes de modificar, salvo que el usuario lo desactive;
- **no transcodifica el stream MP3**.

Los backups se guardan en una carpeta `.nq4-backup` junto a los MP3.

## Arquitectura

MVC y SOLID:

```
View (eframe / egui)
        |
        v
MainController
        |
        +--> TagRepository ------> Id3TagRepository
        +--> FileScanner --------> Mp3FileScanner
        +--> AudioNormalizer ----> IpodSafeNormalizer
        +--> BackupService ------> FileBackupService
        |
        v
Model
(AudioFile, TagData, Diagnostics, NormalizationOptions)
```

La vista no manipula ID3 directamente. El controlador coordina los casos de uso y depende de abstracciones (`traits`). Las implementaciones concretas pueden sustituirse sin modificar la UI.

## Portable

El release está configurado para enlazar el CRT de Windows de forma estática:

```bash
cargo build --release
```

Ejecutable esperado:

```
target/release/normalizador-nq4.exe
```

No requiere instalador para su funcionamiento normal.

## Estado

Aplicación funcional en desarrollo. La implementación se sube al repositorio antes de realizar compilación o validación final, conforme al flujo del proyecto.
