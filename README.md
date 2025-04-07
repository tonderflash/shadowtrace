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

# README - Solución para la Interacción con Teclado en ShadowTrace TUI

## Problema Resuelto

ShadowTrace TUI presentaba problemas con la captura de eventos de teclado, lo que impedía la navegación normal por la interfaz. La aplicación mostraba correctamente los elementos visuales, pero no respondía a las pulsaciones de teclas.

## Descripción de la Solución

Se identificó que el problema estaba en la implementación del sistema de eventos en `src/ui/events.rs`. El código original utilizaba un enfoque no bloqueante (`try_recv()`) que no esperaba adecuadamente por los eventos de teclado.

Las modificaciones implementadas incluyen:

1. **Cambio en el método de lectura de eventos**:

   - Adición de `event::poll()` con un timeout de 100ms antes de intentar leer eventos
   - Esto evita intentos constantes de lectura cuando no hay eventos disponibles

2. **Mejora en el manejo de eventos recibidos**:
   - Reemplazo de `try_recv()` por `recv_timeout()` con un timeout de 50ms
   - Esto permite esperar brevemente por eventos sin bloquear completamente el hilo

## Código Modificado

```rust
// Hilo para eventos de entrada
thread::spawn(move || {
    loop {
        // Leer eventos de manera bloqueante
        if event::poll(Duration::from_millis(100)).unwrap_or(false) {
            if let Ok(event) = event::read() {
                if let Err(err) = event_tx.send(Event::Input(event)) {
                    eprintln!("Error enviando evento: {:?}", err);
                    break;
                }
            }
        }
    }
});

// ...

pub fn next(&self) -> Result<Option<CEvent>> {
    // Usar recv_timeout en lugar de try_recv para bloquear pero con un tiempo límite
    match self.rx.recv_timeout(Duration::from_millis(50)) {
        Ok(Event::Input(event)) => Ok(Some(event)),
        Ok(Event::Tick) => Ok(None),
        Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(anyhow::anyhow!("Canal de eventos desconectado")),
    }
}
```

## Análisis Técnico

### Problema Original

El problema se originaba en la implementación del sistema de eventos que utilizaba `try_recv()`, un método no bloqueante que retorna inmediatamente si no hay eventos disponibles. Esto provocaba que muchos eventos de teclado se perdieran o no fueran procesados correctamente.

### Impacto de los Cambios

- **Eficiencia mejorada**: Reducción del uso de CPU al evitar polling constante
- **Mayor capacidad de respuesta**: Mejor captura de eventos de teclado al esperar activamente
- **Comportamiento más predecible**: Timeouts configurables para adaptar a diferentes entornos

### Entorno de Ejecución

Esta solución ha sido probada en entornos macOS, pero debería funcionar en cualquier plataforma compatible con Rust y Crossterm.

## Uso de la Interfaz

### Teclas de Navegación

- `p` - Acceso al Monitor de Procesos
- `f` - Acceso al Monitor de Archivos
- `n` - Acceso al Monitor de Red
- `r` - Acceso a Reportes
- `h` - Mostrar Ayuda
- `q` o `Esc` - Salir o Volver al Menú Principal

### Teclas Específicas por Pantalla

- **Monitor de Procesos**:

  - Flechas Arriba/Abajo - Navegar entre procesos
  - Enter - Seleccionar proceso para monitoreo
  - `r` - Refrescar lista

- **Otras Pantallas**:
  - Esc - Volver al Dashboard

## Solución de Problemas Adicionales

Si aún experimentas problemas con la entrada de teclado:

1. **Terminal con Foco**: Asegúrate de que la ventana de terminal tiene el foco
2. **Drivers de Teclado**: Verifica que no hay conflictos con software que pueda estar capturando teclas
3. **Variables de Entorno**: Ejecuta con `TERM=xterm-256color cargo run` para forzar un tipo de terminal específico
4. **Terminal Alternativa**: Prueba con un emulador de terminal diferente (iTerm2, Alacritty, etc.)

## Notas Técnicas

Esta implementación utiliza las siguientes bibliotecas:

- **crossterm**: Para captura de eventos y manejo de terminal
- **ratatui**: Para renderizado de la interfaz TUI
- **tokio**: Para el runtime asíncrono

El manejo de eventos se realiza mediante hilos separados que comunican eventos a través de canales MPSC (Multiple Producer, Single Consumer) de Rust, permitiendo una arquitectura desacoplada.

## Referencias

- [Documentación de crossterm](https://docs.rs/crossterm)
- [Documentación de ratatui](https://docs.rs/ratatui)
- [FAQs de ratatui](https://ratatui.rs/faq/)
