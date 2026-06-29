# Taro

**Gestor de integraciones MCP para macOS** — instala, configura y sincroniza integraciones entre Cursor, Claude Desktop y más, sin editar JSON manualmente.

## Características (v0.1)

- **Descubrir** — catálogo de integraciones con instalación guiada
- **Instaladas** — activar, desactivar, eliminar y comprobar integraciones
- **Clientes** — detección automática de Cursor y Claude Desktop
- **Secretos** — credenciales en el Llavero de macOS (nunca en SQLite ni en disco)
- **Salud** — comprobación básica de conectividad de servidores MCP

### Integraciones disponibles

| Integración | Secretos |
|-------------|----------|
| Sistema de archivos | Ninguno |
| Brave Search | Brave API Key |
| GitHub | Token de acceso personal |

Notion, Slack y PostgreSQL aparecen en el catálogo como **Próximamente**.

## Requisitos

- macOS (v0.1 solo macOS)
- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) 18+
- Dependencias opcionales según integración: `node` (npx), `uv`, `brew`

## Desarrollo

```bash
npm install
npm run tauri dev
```

## Compilar

```bash
npm run tauri build
```

El binario se genera en `src-tauri/target/release/bundle/`.

## Arquitectura

```
src/           → React + TypeScript + Tailwind (UI en español)
src-tauri/     → Rust: adaptadores, motor de instalación, Keychain, SQLite
```

### Clientes soportados

| Cliente | Archivo de configuración |
|---------|-------------------------|
| Cursor | `~/.cursor/mcp.json` |
| Claude Desktop | `~/Library/Application Support/Claude/claude_desktop_config.json` |

Las entradas gestionadas por Taro usan claves `taro-{integration_id}`. Se crea una copia de seguridad `*.taro-backup` antes de cada escritura.

### Secretos

- Servicio Keychain: `taro`
- Cuenta: `{integration_id}:{secret_key}`

## Seguridad

- Los secretos solo viven en el Llavero de macOS
- Copias de seguridad automáticas antes de modificar configuraciones de clientes
- Validación JSON y rollback en caso de error de escritura

## Fuera de alcance (v0.1)

- Perfiles, Smart Import completo, adaptadores Codex/VS Code
- OAuth, auto-actualización de paquetes MCP
- Windows/Linux, distribución App Store / notarización

## Licencia

MIT
