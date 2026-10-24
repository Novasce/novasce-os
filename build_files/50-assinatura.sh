#!/usr/bin/bash
# A máquina só aplica a imagem da Novasce assinada com a chave da casa.
# A política do Aurora é mesclada, não trocada: ghcr.io/ublue-os continua confiável.
set -euo pipefail

CHAVE=/usr/lib/pki/containers/novasce-os.pub
POLITICA=/etc/containers/policy.json

test -s /ctx/cosign.pub || { echo "Falta o cosign.pub na raiz do repositório (M0, item 2)." >&2; exit 1; }
grep -q 'BEGIN PUBLIC KEY' /ctx/cosign.pub
install -Dm0644 /ctx/cosign.pub "$CHAVE"

jq --arg chave "$CHAVE" '
  .transports.docker["ghcr.io/novasce/novasce-os"] = [{
    "type": "sigstoreSigned",
    "keyPath": $chave,
    "signedIdentity": { "type": "matchRepository" }
  }]' "$POLITICA" >"$POLITICA.novo"
install -m0644 "$POLITICA.novo" "$POLITICA"
rm "$POLITICA.novo"

jq -e '.default[0].type == "reject"' "$POLITICA" >/dev/null
jq -e '.transports.docker["ghcr.io/ublue-os"][0].type == "sigstoreSigned"' "$POLITICA" >/dev/null
jq -e '.transports.docker["ghcr.io/novasce/novasce-os"][0].keyPath == "/usr/lib/pki/containers/novasce-os.pub"' "$POLITICA" >/dev/null
