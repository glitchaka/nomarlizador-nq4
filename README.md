# Normalizador NQ4

Aplicación de escritorio portable en Rust para inspeccionar, crear, editar y normalizar etiquetas ID3 de archivos MP3.

## Objetivo inicial

El perfil **iPod seguro** reescribe únicamente los metadatos, sin recodificar el audio:

- ID3v2.3.
- elimina `TLEN` (evita duraciones incorrectas heredadas);
- opcionalmente elimina ID3v1 y APEv2 finales;
- conserva los campos comunes: título, artista, álbum, género, año, pista, disco y comentario;
- conserva la portada o la reescribe como JPEG compatible y limitada a 600x600;
- puede crear una copia de seguridad antes de modificar;
- el audio MP3 no se transcodifica.

## Interfaz

- agregar uno o varios MP3;
- agregar una carpeta completa;
- arrastrar MP3 a la ventana;
- inspeccionar versión ID3, TLEN, ID3v1/APEv2 y portada;
- editar y crear tags;
- cambiar o eliminar portada;
- guardar solo los cambios del archivo seleccionado;
- normalizar el archivo seleccionado o todos los cargados.

## Arquitectura

La aplicación está separada en MVC y aplica SOLID mediante composición y traits:

```
View (eframe/egui)
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

La vista no conoce detalles de ID3 ni manipula archivos. El controlador coordina casos de uso. Los servicios se consumen mediante interfaces (`traits`) sustituibles.

## Portable

La configuración de release intenta enlazar el CRT de Windows de forma estática. El ejecutable final se generará en `target/release/normalizador-nq4.exe`.

## Estado

Primera implementación funcional. Por regla del proyecto, esta tanda fue escrita y subida antes de compilar o ejecutar pruebas.
