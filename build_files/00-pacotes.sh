#!/usr/bin/bash
# Pacotes que a Novasce acrescenta ao Aurora.
#  plymouth-plugin-script e plymouth-scripts: tema de boot próprio (20-plymouth.sh)
#  librsvg2-tools: imagens da marca a partir do vetor oficial, e o avatar das contas
#  rsms-inter-fonts: a fonte da marca (Inter) no sistema e nos apps da casa
#  webkit2gtk4.1: o motor de tela do Preparar (Tauri)
#  pciutils: lspci, para o Preparar achar a placa NVIDIA
#  nodejs, npm e git: o executor dos agentes na sede (Node 22 ou mais)
set -euo pipefail

dnf5 install -y --setopt=install_weak_deps=False \
  plymouth-plugin-script plymouth-scripts librsvg2-tools rsms-inter-fonts \
  webkit2gtk4.1 pciutils nodejs npm git
node -e 'if (+process.versions.node.split(".")[0] < 22) { console.error("Node < 22"); process.exit(1) }'
fc-list : family | grep -q '^Inter'
dnf5 clean all
