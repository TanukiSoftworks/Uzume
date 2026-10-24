<div align="center">

<img src="./assets/icon.png" alt="Uzume Logo" width="128" height="128" />

# Uzume

**Cliente nativo de Discord Rich Presence para Windows mediante SMTC**

[![Licencia: GPL-3.0-or-later](https://img.shields.io/badge/licencia-GPL--3.0--or--later-blue.svg)](https://www.gnu.org/licenses/gpl-3.0.html)
[![Plataforma: Windows 10/11 x64](https://img.shields.io/badge/plataforma-Windows%2010%20%2F%2011%20(x64)-informational.svg)](https://github.com/TanukiSoftworks/Uzume)
[![Lenguaje: Rust](https://img.shields.io/badge/lenguaje-Rust%20(Win32%20%2F%20GDI%2B)-orange.svg)](https://www.rust-lang.org)

</div>

---

## Descripcion

Uzume es un servicio de escritorio para Windows que conecta el subsistema de transporte multimedia del sistema operativo (**System Media Transport Controls / SMTC**) con el **Rich Presence de Discord**.

A diferencia de soluciones construidas sobre entornos empaquetados como Electron o dependientes de extensiones de navegador, Uzume esta programado integramente en Rust contra las APIs nativas de Windows (Win32, GDI+ y WinRT). Se ejecuta con un consumo de memoria residente menor a 30 MB, no requiere navegadores embebidos, se comunica de forma estrictamente local mediante Named Pipes y no recopila telemetria ni datos de uso.

Desarrollado y mantenido por **Tanuki Softworks**.

---

## Caracteristicas principales

- **Intercepcion nativa de SMTC:** Lee directamente las sesiones de audio expuestas por Windows. Si hay multiples fuentes abiertas, prioriza automaticamente la pista con reproduccion activa frente a las instancias pausadas o en silencio.
- **Filtro de procesos del sistema:** Ignora de forma deliberada navegadores web convencionales (Chrome, Firefox, Edge, Brave) para impedir que llamadas de voz, pestañas en segundo plano o videos aleatorios interrumpan la presencia musical.
- **Resolucion de caratulas:** Extrae las miniaturas provistas por el propio reproductor o consulta caratulas de alta definicion (1000x1000) en catalogos publicos, almacenando en cache los resultados para evitar consultas redundantes de red.
- **Modos de estado configurables:**
  - *Titulo de la cancion:* Muestra el nombre de la pista activa en el estado del perfil.
  - *Nombre del reproductor:* Refleja la aplicacion origen detectada por SMTC.
  - *Texto personalizado:* Permite definir un mensaje estatico fijo (por ejemplo, sesiones de concentracion, estudio o trabajo).
- **Interfaz grafica ligera en Win32 y GDI+:** Ventana de configuracion con paleta sobria, soporte de tema oscuro mediante DWM en Windows 10 y 11, y minimizacion al area de notificaciones (System Tray).
- **Localizacion en 6 idiomas:** Integra traducciones completas incrustadas en el ejecutable (español, ingles, japones, coreano, chino simplificado y chino tradicional) con deteccion automatica segun el idioma del sistema.
- **Comunicacion local y tolerancia a fallos:** Se comunica con el cliente de Discord exclusivamente mediante pipes locales de Windows (`\\.\pipe\discord-ipc-0`) y cuenta con manejo estructurado de excepciones para prevenir cierres involuntarios ante desconexiones de audio.

---

## Reproductores compatibles

Cualquier reproductor o aplicacion que reporte su reproduccion a Windows SMTC es reconocido de inmediato:

- Spotify (Desktop y Tienda de Windows)
- Deezer (Desktop y Tienda de Windows)
- Apple Music (Aplicacion para Windows)
- Tidal (Desktop Hi-Fi)
- YouTube Music Desktop
- Amazon Music
- foobar2000 (con soporte o plugin de SMTC)
- MusicBee
- AIMP
- VLC Media Player

---

## Comparativa tecnica de recursos

| Parametro tecnico | Uzume (Rust + Win32) | Clientes basados en Electron / Web |
| :--- | :--- | :--- |
| **Motor de ejecucion** | Rust nativo (Win32 / GDI+) | Node.js + Chromium embebido |
| **Memoria de trabajo (RAM)** | 26 a 32 MB | 220 a 480 MB |
| **Tamaño del instalador** | ~3.1 MB | 75 a 140 MB |
| **Tamaño del binario en disco** | ~3.0 MB | 180 a 220 MB |
| **Uso de CPU en reposo** | 0.0% (sondeo pasivo a 1s) | 1.5% a 4.5% (bucle de renderizado web) |
| **Canal de comunicacion** | Named Pipes locales directos | Puentes intermedios o WebSockets |
| **Telemetria / Analitica** | Ninguna (100% local) | Recoleccion de eventos y metricas remotas |

---

## Compilacion desde codigo fuente

### Requisitos

- Windows 10 o Windows 11 (x64)
- Rust toolchain estable (version 1.75 o superior) con el target `x86_64-pc-windows-msvc`
- Herramientas de compilacion de C++ (Visual Studio Build Tools)
- *(Opcional para empaquetado)* Inno Setup 6.x

### Compilar el binario

```powershell
# Clonar o ubicarse en el directorio del proyecto
cd K:\athena\Uzume

# Compilar binario optimizado para release
cargo build --release
```

El ejecutable autosuficiente se generara en:
```text
target/release/uzume.exe
```

### Generar el instalador (Inno Setup)

```powershell
& "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer\uzume_setup.iss
```

El instalador por usuario sin requerimiento de privilegios de administrador se guardara en:
```text
..\releases\Uzume-Setup-x64.exe
```

---

## Estructura del proyecto

```text
Uzume/
├── assets/
│   ├── icon.ico              # Icono de la aplicacion en formato ICO
│   ├── icon.png              # Icono en formato PNG
│   └── icon.svg              # Grafico vectorial en SVG
├── installer/
│   ├── Languages/            # Archivos de idioma para Inno Setup
│   ├── notice_*.txt          # Avisos de traduccion generada por IA
│   └── uzume_setup.iss       # Script de configuracion de Inno Setup
├── locales/
│   ├── en.json               # Ingles
│   ├── es.json               # Español
│   ├── ja.json               # Japones
│   ├── ko.json               # Coreano
│   ├── zh-hans.json          # Chino simplificado
│   └── zh-hant.json          # Chino tradicional
├── src/
│   ├── config.rs             # Carga y persistencia del perfil de usuario en JSON
│   ├── cover.rs              # Resolucion de portadas en alta definicion y cache local
│   ├── discord.rs            # Comunicacion IPC con Discord mediante Named Pipes
│   ├── i18n.rs               # Deteccion regional e incrustacion de cadenas
│   ├── logger.rs             # Registro rotativo de eventos en disco
│   ├── main.rs               # Coordinador del ciclo de vida y bucle SMTC
│   ├── smtc.rs               # Cliente WinRT para lectura del transporte multimedia
│   └── ui.rs                 # Ventana de configuracion Win32, GDI+ y System Tray
├── build.rs                  # Incrustacion de recursos de version e icono PE
├── Cargo.toml                # Dependencias y metadatos del paquete
└── README.md                 # Documentacion tecnica del repositorio
```

---

## Licencia

Este proyecto esta publicado como software libre bajo los terminos de la **GNU General Public License v3.0 o superior (GPL-3.0-or-later)**. Consulta el archivo de licencia o la pagina del proyecto para mas detalles.

```text
Uzume — Windows SMTC Discord Rich Presence Client
Copyright (C) 2025-2026 Tanuki Softworks <luis.sepulveda@tanukisoftworks.com>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.
```
