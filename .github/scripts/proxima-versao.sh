#!/usr/bin/env bash
# Recebe as tags já publicadas na entrada (uma por linha) e a data AAAA.MM.DD.
# Imprime a próxima versão do dia: AAAA.MM.DD.N. Só conta tags N-nvidia e N-padrao.
set -euo pipefail

HOJE=${1:?passe a data AAAA.MM.DD}
if [[ ! $HOJE =~ ^[0-9]{4}\.[0-9]{2}\.[0-9]{2}$ ]]; then
  echo "data inválida: $HOJE" >&2
  exit 1
fi

maior=0
while IFS= read -r tag; do
  [[ $tag == "$HOJE".* ]] || continue
  resto=${tag#"$HOJE".}
  numero=${resto%-*}
  variante=${resto##*-}
  [[ $numero =~ ^[0-9]+$ ]] || continue
  [[ $variante == nvidia || $variante == padrao ]] || continue
  # 10# evita que um número com zero à esquerda seja lido como octal.
  if ((10#$numero > maior)); then
    maior=$((10#$numero))
  fi
done
echo "${HOJE}.$((maior + 1))"
