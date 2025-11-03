# ShadowTrace 🕵️‍♂️

**An AI-powered black-box style debugger**

ShadowTrace is a local tool that analyzes processes, files, and network connections in real time, generating explanatory reports with the help of language models (LLMs). It helps you understand the behavior of opaque binaries without access to their original source code.

## 🔍 Features

- Intercepts processes executed on the system
- Monitors opened files (read/write)
- Detects network connections (sockets, IPs, ports)
- Produces structured JSON logs
- Uses a local LLM to analyze patterns and generate explanations
- Command-line interface (CLI)

## 🚀 Installation

### Automated (recommended)

```bash
# Clone the repository
git clone https://github.com/usuario/shadowtrace.git
cd shadowtrace

# Run the installer script
./install.sh
```

The installer checks dependencies like Rust and Ollama (optional), then builds and installs ShadowTrace.

### Manual installation

```bash
# Clone the repository
git clone https://github.com/usuario/shadowtrace.git
cd shadowtrace

# Build
cargo build --release

# Optional: install globally
sudo cp target/release/shadowtrace /usr/local/bin/
```

### Prerequisites

- **Rust** (1.70+): Required to build the project
- **Ollama** (recommended): For LLM analysis functionality
  - Recommended models: llama2, mistral, orca-mini

## 📋 Usage

```bash
# Help and available options
shadowtrace --help

# Monitor a specific process
shadowtrace monitor --pid 1234
shadowtrace monitor --name firefox --duration 120

# Audit a binary
shadowtrace audit --binary /path/to/binary

# Monitor all system processes
shadowtrace system --watch

# Use a specific model
shadowtrace --model mistral monitor --name chrome
```

## 📊 Reports

ShadowTrace automatically generates detailed reports in JSON and Markdown formats, stored at:

```
~/.shadowtrace/reports/
```

Reports include:

- Comprehensive process information
- Detected file events
- Established network connections
- Detailed LLM analysis
- Alerts and warnings

## 🛠️ Technologies

- Rust for performance and safety
- Ollama/llama.cpp for local LLM processing
- Clap for command-line interface

## ⚠️ Current limitations

- Real interception of file and network operations is in progress
- Binary audit mode is partially implemented
- Detecting certain suspicious behaviors may require elevated privileges

## 🧩 Contributing

Contributions are welcome! Check the [issues](https://github.com/usuario/shadowtrace/issues) to get started.

## 📄 License

This project is licensed under the MIT License.

# README - Fix for Keyboard Input in ShadowTrace TUI

## Resolved Problem

ShadowTrace TUI had issues capturing keyboard events, preventing normal navigation. The UI rendered correctly but did not respond to key presses.

## Solution Description

The issue was identified in the event system implementation in `src/ui/events.rs`. The original code used a non-blocking approach (`try_recv()`) that did not wait properly for keyboard events.

Changes implemented:

1. **Event reading method change**:

   - Added `event::poll()` with a 100 ms timeout before attempting to read events
   - This avoids constant read attempts when there are no events

2. **Improved handling of received events**:
   - Replaced `try_recv()` with `recv_timeout()` with a 50 ms timeout
   - This allows briefly waiting for events without fully blocking the thread

## Modified Code

```rust
// Input event thread
thread::spawn(move || {
    loop {
        // Blockingly read events
        if event::poll(Duration::from_millis(100)).unwrap_or(false) {
            if let Ok(event) = event::read() {
                if let Err(err) = event_tx.send(Event::Input(event)) {
                    eprintln!("Error sending event: {:?}", err);
                    break;
                }
            }
        }
    }
});

// ...

pub fn next(&self) -> Result<Option<CEvent>> {
    // Use recv_timeout instead of try_recv to block with a time limit
    match self.rx.recv_timeout(Duration::from_millis(50)) {
        Ok(Event::Input(event)) => Ok(Some(event)),
        Ok(Event::Tick) => Ok(None),
        Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(anyhow::anyhow!("Event channel disconnected")),
    }
}
```

## Technical Analysis

### Original Problem

The problem originated in the event system implementation using `try_recv()`, a non-blocking method that returns immediately if there are no events. This caused many key events to be missed or not processed correctly.

### Impact of the Changes

- **Improved efficiency**: Reduced CPU usage by avoiding constant polling
- **Better responsiveness**: Improved key event capture by actively waiting
- **More predictable behavior**: Configurable timeouts for different environments

### Runtime Environment

This solution has been tested on macOS, and should work on any platform supported by Rust and Crossterm.

## Interface Usage

### Navigation Keys

- `p` - Process Monitor
- `f` - File Monitor
- `n` - Network Monitor
- `r` - Reports
- `h` - Help
- `q` or `Esc` - Exit or Return to Main Menu

### Screen-Specific Keys

- **Process Monitor**:

  - Up/Down arrows - Navigate between processes
  - Enter - Select a process for monitoring
  - `r` - Refresh list

- **Other Screens**:
  - Esc - Return to Dashboard

## Additional Troubleshooting

If you still experience keyboard input issues:

1. **Terminal focus**: Ensure the terminal window is focused
2. **Keyboard drivers**: Check for software conflicts that might capture keys
3. **Environment variables**: Run with `TERM=xterm-256color cargo run` to force a specific terminal type
4. **Alternate terminal**: Try a different terminal emulator (iTerm2, Alacritty, etc.)

## Technical Notes

This implementation uses the following libraries:

- **crossterm**: Event capture and terminal handling
- **ratatui**: TUI rendering
- **tokio**: Async runtime

Event handling uses separate threads with Rust MPSC channels (Multiple Producer, Single Consumer), enabling a decoupled architecture.

## References

- [crossterm documentation](https://docs.rs/crossterm)
- [ratatui documentation](https://docs.rs/ratatui)
- [ratatui FAQs](https://ratatui.rs/faq/)
