# ShadowTrace 🕵️‍♂️

**Un debugger tipo caja negra potenciado por IA**

ShadowTrace es una herramienta local que analiza procesos, archivos y conexiones de red en tiempo real, generando reportes explicativos con ayuda de modelos de lenguaje (LLM). Permite entender el comportamiento de binarios opacos sin necesidad de acceder al código fuente original.

## 🔍 Características

- Intercepta procesos ejecutados en el sistema
- Monitorea archivos abiertos (lectura/escritura)
- Detecta conexiones de red (sockets, IPs, puertos)
- Genera logs estructurados en JSON
- Utiliza LLM local para analizar patrones y generar explicaciones
- Interfaz por línea de comandos (CLI)

## 🚀 Instalación

### Método automatizado (recomendado)

```bash
# Clonar el repositorio
git clone https://github.com/usuario/shadowtrace.git
cd shadowtrace

# Ejecutar el script de instalación
./install.sh
```

El script de instalación verificará las dependencias necesarias, como Rust y Ollama (opcional), y compilará e instalará ShadowTrace.

### Instalación manual

```bash
# Clonar el repositorio
git clone https://github.com/usuario/shadowtrace.git
cd shadowtrace

# Compilar
cargo build --release

# Opcional: Instalar globalmente
sudo cp target/release/shadowtrace /usr/local/bin/
```

### Prerrequisitos

- **Rust** (1.70+): Necesario para compilar el proyecto
- **Ollama** (recomendado): Para la funcionalidad de análisis con LLM
  - Modelos recomendados: llama2, mistral, orca-mini

## 📋 Uso

```bash
# Ver ayuda y opciones disponibles
shadowtrace --help

# Monitorear un proceso específico
shadowtrace monitor --pid 1234
shadowtrace monitor --name firefox --duration 120

# Analizar un binario
shadowtrace audit --binary /path/to/binary

# Monitorear todos los procesos del sistema
shadowtrace system --watch

# Usar un modelo específico
shadowtrace --model mistral monitor --name chrome
```

## 📊 Reportes

ShadowTrace genera automáticamente reportes detallados en formatos JSON y Markdown. Estos se guardan en:

```
~/.shadowtrace/reports/
```

Los reportes incluyen:

- Información completa del proceso
- Eventos de archivo detectados
- Conexiones de red establecidas
- Análisis detallado del LLM
- Alertas y advertencias detectadas

## 🛠️ Tecnologías

- Rust para rendimiento y seguridad
- Ollama/llama.cpp para procesamiento LLM local
- Clap para la interfaz de línea de comandos

## ⚠️ Limitaciones actuales

- La interceptación real de operaciones de archivo y red está en desarrollo
- El modo de auditoría de binarios está parcialmente implementado
- Algunos comportamientos sospechosos pueden requerir permisos elevados para su detección

## 🧩 Contribuir

Contribuciones son bienvenidas! Ve a la sección de [issues](https://github.com/usuario/shadowtrace/issues) para comenzar.

## 📄 Licencia

Este proyecto está licenciado bajo MIT License.
