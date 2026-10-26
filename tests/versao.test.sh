#!/usr/bin/env bash
# Testa a conta da versão do dia (.github/scripts/proxima-versao.sh).
set -euo pipefail

SCRIPT="$(dirname "$0")/../.github/scripts/proxima-versao.sh"

esperar() {
  local nome=$1 esperado=$2 obtido=$3
  if [ "$esperado" = "$obtido" ]; then
    echo "ok    $nome"
  else
    echo "FALHA $nome: esperado $esperado, veio $obtido"
    exit 1
  fi
}

esperar "primeira do dia" 2026.10.24.1 "$(printf '' | bash "$SCRIPT" 2026.10.24)"
esperar "segunda do dia" 2026.10.24.2 "$(printf '2026.10.24.1-nvidia\n2026.10.24.1-padrao\nstable-nvidia\n' | bash "$SCRIPT" 2026.10.24)"
esperar "ignora outros dias" 2026.10.24.1 "$(printf '2026.10.23.5-padrao\n' | bash "$SCRIPT" 2026.10.24)"
esperar "pega a maior" 2026.10.24.4 "$(printf '2026.10.24.3-padrao\n2026.10.24.1-nvidia\n' | bash "$SCRIPT" 2026.10.24)"
esperar "ignora tag estranha" 2026.10.24.1 "$(printf '2026.10.24.x-padrao\n2026.10.24.2-outra\n' | bash "$SCRIPT" 2026.10.24)"
