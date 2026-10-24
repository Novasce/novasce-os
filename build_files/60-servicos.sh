#!/usr/bin/bash
# Serviços que nascem ligados em toda máquina. O executor dos agentes NÃO entra
# aqui: ele liga na sede, depois do doctor (novasce-primeiro-uso).
set -euo pipefail

systemctl enable novasce-sinal.timer
