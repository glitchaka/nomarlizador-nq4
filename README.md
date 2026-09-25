# Normalizador NQ4

Aplicación de escritorio portable para Windows, escrita en Rust, para inspeccionar, crear, editar y normalizar etiquetas ID3 de MP3 sin recodificar el audio.

## Interfaz

La interfaz utiliza una composición de aplicación de escritorio moderna:

- ventana sin decoración nativa;
- barra superior propia arrastrable;
- controles de minimizar, maximizar/restaurar y cerrar a la derecha;
- navegación lateral;
- página de Resumen con tarjetas;
- Biblioteca con tabla completa de MP3;
- página de Normalización;
- carátula visible directamente en el resumen y la selección;
- edición de tags en ventana independiente;
- diagnóstico, preferencias y vista ampliada de carátula en ventanas independientes;
- progreso de normalización por lote;
- búsqueda y drag & drop.

## Flujo

1. Añadir MP3 o una carpeta completa.
2. Revisar carátulas, tags y alertas.
3. Editar solo cuando sea necesario.
4. Usar **Normalizar todos** para procesar la biblioteca completa.
5. Los resultados se guardan en **MP3 normalizados**.

Los originales se conservan y el audio MP3 no se recodifica.

## Perfil iPod seguro

- ID3v2.3;
- elimina TLEN;
- puede eliminar ID3v1 y APEv2;
- puede convertir la carátula a JPEG compatible;
- salida en **MP3 normalizados**.

## Arquitectura

MVC + SOLID.

## Compilación

```powershell
cargo build --release
```

Ejecutable:

```
target\release\normalizador-nq4.exe
```
