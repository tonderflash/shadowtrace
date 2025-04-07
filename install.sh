#!/bin/bash

# Script de instalación para ShadowTrace

# Colores para salida
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== ShadowTrace Instalador ===${NC}"
echo "Esto instalará ShadowTrace y sus dependencias."

# Verificar si Rust está instalado
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Rust no está instalado. Instalando rustup...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo -e "${GREEN}✓ Rust ya está instalado${NC}"
fi

# Verificar si Ollama está instalado (opcional)
if ! command -v ollama &> /dev/null; then
    echo -e "${YELLOW}Ollama no está instalado. Se recomienda instalarlo para la funcionalidad LLM.${NC}"
    echo "Puedes instalarlo desde: https://ollama.ai"
    
    read -p "¿Quieres intentar instalar Ollama ahora? (s/n): " install_ollama
    if [[ $install_ollama == "s" || $install_ollama == "S" ]]; then
        echo "Instalando Ollama..."
        curl -fsSL https://ollama.ai/install.sh | sh
    else
        echo "Continuando sin instalar Ollama. Puedes usar la opción --no-llm al ejecutar ShadowTrace."
    fi
else
    echo -e "${GREEN}✓ Ollama ya está instalado${NC}"
    
    # Verificar si hay modelos instalados
    if ! ollama list | grep -q "llama"; then
        echo -e "${YELLOW}No se detectaron modelos de Ollama. Se recomienda instalar al menos uno.${NC}"
        read -p "¿Quieres descargar el modelo llama2 ahora? (s/n): " install_model
        if [[ $install_model == "s" || $install_model == "S" ]]; then
            echo "Descargando modelo llama2..."
            ollama pull llama2
        fi
    else
        echo -e "${GREEN}✓ Modelos de Ollama detectados${NC}"
    fi
fi

# Construir el proyecto
echo "Compilando ShadowTrace..."
cargo build --release

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Compilación exitosa${NC}"
    
    # Crear directorio para reportes
    mkdir -p ~/.shadowtrace/reports
    
    # Copiar el binario a un directorio en el PATH
    read -p "¿Quieres instalar ShadowTrace en /usr/local/bin? (s/n): " install_bin
    if [[ $install_bin == "s" || $install_bin == "S" ]]; then
        echo "Instalando ShadowTrace en /usr/local/bin..."
        sudo cp target/release/shadowtrace /usr/local/bin/
        echo -e "${GREEN}✓ ShadowTrace instalado correctamente${NC}"
        echo "Puedes ejecutarlo con el comando 'shadowtrace'"
    else
        echo -e "${YELLOW}ShadowTrace no se ha instalado globalmente.${NC}"
        echo "Puedes ejecutarlo con './target/release/shadowtrace'"
    fi
    
    # Instrucciones finales
    echo -e "\n${GREEN}=== Instalación completada ===${NC}"
    echo "Para ver las opciones disponibles ejecuta: shadowtrace --help"
    echo "Ejemplos de uso:"
    echo "  shadowtrace monitor --name firefox"
    echo "  shadowtrace system --watch"
    echo -e "${YELLOW}Nota: Algunas funcionalidades requieren permisos de root/administrador${NC}"
else
    echo -e "${RED}✗ Error durante la compilación${NC}"
    exit 1
fi 
