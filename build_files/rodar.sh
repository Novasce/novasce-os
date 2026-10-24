#!/usr/bin/bash
# Roda os passos da montagem, na ordem do número. Um arquivo por assunto.
# Recebe VARIANTE (nvidia|padrao) e IMAGE_VERSION (AAAA.MM.DD.N) do Containerfile.
set -euo pipefail

case "${VARIANTE:-}" in
  nvidia | padrao) ;;
  *) echo "VARIANTE inválida: '${VARIANTE:-}'" >&2; exit 1 ;;
esac
if [[ ! ${IMAGE_VERSION:-} =~ ^[0-9]{4}\.[0-9]{2}\.[0-9]{2}\.[0-9]+$ ]]; then
  echo "IMAGE_VERSION fora do formato AAAA.MM.DD.N: '${IMAGE_VERSION:-}'" >&2
  exit 1
fi
export VARIANTE IMAGE_VERSION

for passo in /ctx/build_files/[0-9][0-9]-*.sh; do
  echo "::group:: ${passo##*/}"
  bash "$passo"
  echo "::endgroup::"
done
