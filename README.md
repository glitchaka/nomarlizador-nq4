# Normalizador NQ4

Aplicación de escritorio portable para Windows, escrita en Rust, para inspeccionar, crear, editar y normalizar etiquetas ID3 de MP3 sin recodificar el audio.

## Flujo principal

1. Añade MP3 individuales o una carpeta completa.
2. Revisa las alertas ID3.
3. Edita tags desde la ventana **Editar**.
4. Normaliza uno o varios archivos.
5. Los resultados se escriben en **MP3 normalizados**, junto a la carpeta de origen.

La normalización **no modifica el MP3 original**.

## Interfaz

- ventana sin decoración nativa;
- barra superior propia, arrastrable;
- controles propios de minimizar y cerrar;
- menús Archivo, Editar, Herramientas, Ver y Ayuda;
- barra de acciones reordenada: añadir, editar, normalizar, lote y abrir salida;
- panel de archivos y buscador;
- vista principal de resumen;
- edición de tags en su propia ventana;
- ventanas separadas para Preferencias, Diagnóstico, Portada y Acerca de;
- progreso para normalización por lote;
- drag & drop y atajos.

## Edición ID3

Permite editar título, artista, álbum, género, año, pista, disco, comentario y portada.

## Perfil iPod seguro

- salida en **MP3 normalizados**;
- ID3v2.3;
- elimina TLEN;
- eliminación opcional de ID3v1 y APEv2;
- portada opcionalmente convertida a JPEG con tamaño máximo configurable;
- no transcodifica el stream MP3.

## Arquitectura

MVC + SOLID:

```
View (eframe / egui)
        |
        v
MainController
        |
        +--> TagRepository ------> Id3TagRepository
        +--> FileScanner --------> Mp3FileScanner
        +--> AudioNormalizer ----> IpodSafeNormalizer
        |
        v
Model
(AudioFile, TagData, Diagnostics, NormalizationOptions)
```

## Compilación

```powershell
cargo build --release
```

Ejecutable:

```
target\release\normalizador-nq4.exe
```
