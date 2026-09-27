# Reglas de Desarrollo del Proyecto (World of Azeria)

## 1. Control de Publicación en GitHub
- **NUNCA** hacer `git push`, crear tags de release ni publicar en GitHub de forma automática.
- **SOLO** enviar a GitHub cuando el usuario lo solicite explícitamente (ej: "sube los cambios", "publica en github", "haz push").

## 2. Control de Compilación
- **NUNCA** ejecutar compilaciones pesadas (`cargo build --release`, compilación del cliente o servidor) de forma automática.
- Para verificar errores de código y tipos durante el desarrollo, utilizar únicamente `cargo check`.
- **SOLO** compilar cuando el usuario lo indique explícitamente (ej: "compila el juego", "compila la versión", "haz build").

## 3. Registro Continuo en la Web (Logs / Changelog)
- **SIEMPRE** registrar cada cambio, mejora, corrección o nueva característica implementada en los archivos de la web:
  - `website/CHANGELOG.txt`
  - `website/index.html` (sección de novedades / notas de versión)
