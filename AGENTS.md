# AGENTS.md

Instrucciones para cualquier agente que trabaje en este repositorio.

Este archivo adapta las pautas de `multica-ai/andrej-karpathy-skills` a un formato neutral para todos los agentes. La idea central es reducir errores comunes de agentes de codigo: asumir demasiado, sobrecomplicar, tocar codigo no relacionado y cerrar tareas sin verificacion.

## Contexto del Proyecto

Taro es una app macOS para gestionar integraciones MCP. La interfaz esta en React + TypeScript + Tailwind y el backend nativo esta en Rust con Tauri 2.

- UI: `src/`
- Backend Tauri/Rust: `src-tauri/`
- Scripts auxiliares: `scripts/`
- El producto esta orientado a macOS y a clientes como Cursor y Claude Desktop.
- La UI del producto esta en espanol; conserva ese idioma en textos visibles salvo que la tarea pida otra cosa.
- Los secretos deben vivir en Keychain. No escribas credenciales en SQLite, JSON, logs, fixtures ni archivos de configuracion.

## Codebase Knowledge Graph

Este proyecto usa `codebase-memory-mcp` para mantener un grafo de conocimiento del codigo. Para descubrir codigo, prefiere siempre las herramientas MCP del grafo antes que busquedas de texto.

Orden de prioridad:

1. `search_graph` para encontrar funciones, clases, rutas, variables o patrones.
2. `trace_path` para ver quien llama a una funcion o que llama esa funcion.
3. `get_code_snippet` para leer el codigo de una funcion o clase concreta.
4. `query_graph` para consultas complejas.
5. `get_architecture` para contexto de alto nivel.

Usa busqueda de texto solo para literales, mensajes de error, configuracion, scripts, documentacion o cuando el grafo no devuelva informacion suficiente.

## 1. Piensa Antes de Codificar

No asumas en silencio. Si una tarea admite varias interpretaciones, nombra la ambiguedad y elige solo cuando sea razonable hacerlo.

- Declara supuestos importantes antes de cambios con impacto.
- Pide aclaracion cuando el coste de equivocarse sea alto.
- Si ves una solucion mas simple o con menos riesgo, dilo.
- Si algo no encaja con la arquitectura existente, detenlo y explica la inconsistencia.

Para tareas pequenas y obvias, usa criterio y evita rituales innecesarios.

## 2. Simplicidad Primero

Implementa el minimo codigo que resuelve el problema actual.

- No anadas funcionalidades que no se pidieron.
- No crees abstracciones para codigo de un solo uso.
- No anadas configurabilidad especulativa.
- No introduzcas dependencias sin una razon concreta.
- Si una solucion empieza a crecer demasiado, simplificala antes de seguir.

Pregunta practica: si un ingeniero senior diria que esta sobrecomplicado, reescribelo mas simple.

## 3. Cambios Quirurgicos

Toca solo lo necesario para la solicitud.

- No refactorices codigo adyacente por preferencia personal.
- No reformatees archivos enteros salvo que la tarea lo requiera.
- Sigue el estilo existente aunque no sea tu estilo preferido.
- Si encuentras deuda tecnica no relacionada, mencionala en el cierre en vez de arreglarla.
- Elimina imports, variables, funciones o archivos que tus propios cambios hayan dejado sin uso.
- No elimines codigo muerto preexistente sin permiso explicito.

Cada linea modificada debe poder justificarse por la solicitud del usuario.

## 4. Ejecucion Orientada a Objetivos

Convierte la tarea en un objetivo verificable y trabaja hasta comprobarlo.

Ejemplos:

- "Arregla el bug" significa reproducirlo o identificar la causa, aplicar el cambio y verificar que ya no ocurre.
- "Anade validacion" significa cubrir entradas invalidas y comprobar el comportamiento esperado.
- "Refactoriza" significa conservar comportamiento y ejecutar verificaciones antes de cerrar.

Para tareas de varios pasos, usa un plan breve con una comprobacion por paso. No termines solo porque el codigo compila en tu cabeza.

## Comandos Habituales

- Instalar dependencias: `npm install`
- Desarrollo web: `npm run dev`
- Desarrollo Tauri: `npm run tauri dev`
- Build frontend: `npm run build`
- Build app: `npm run tauri build`
- Rust checks desde `src-tauri/`: `cargo check`

Antes de cerrar cambios de codigo, ejecuta la verificacion mas especifica y barata que cubra el area tocada. Si no puedes ejecutarla, explica por que.

## Convenciones de Implementacion

- React + TypeScript vive en `src/`; Rust/Tauri vive en `src-tauri/`.
- Mantén tipos compartidos y contratos frontend/backend sincronizados.
- Usa patrones existentes para comandos Tauri, errores, estado de UI y componentes.
- Preserva las garantias de seguridad: backups antes de escribir configuraciones de clientes, validacion JSON y rollback si aplica.
- No registres secretos ni valores sensibles.
- En UI, usa componentes e iconos existentes cuando sea posible y evita cambios visuales no pedidos.

## Mantenimiento de herramientas entre tests

- Borra componentes como uv u otros instalados por mcps tras cada implementación.
- De esa manera el testing será más sencillo

## Criterios de Cierre

Antes de responder como terminado:

- Revisa el diff y confirma que solo incluye cambios relacionados.
- Ejecuta las pruebas o builds relevantes para el area modificada.
- Menciona cualquier verificacion no ejecutada.
- Resume el cambio en terminos de comportamiento, no solo archivos tocados.
- Haz un commit en tu worktree de lo implementado sin pushear a origin ni a la rama principal.

Estas pautas funcionan si los diffs son pequenos, las preguntas de aclaracion aparecen antes de implementar cuando hacen falta, y las tareas terminan con una comprobacion concreta.
