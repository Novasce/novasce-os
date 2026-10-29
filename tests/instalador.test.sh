#!/usr/bin/env bash
# Testa o instalador da casa sem root, sem podman e sem pendrive:
# a configuração do bootc-image-builder (instalador/gerar.sh --so-config) e a escolha
# do aparelho no gravar-pendrive.sh, com um lsblk de mentira.
set -euo pipefail

RAIZ="$(cd "$(dirname "$0")/.." && pwd)"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

ok() { echo "ok    $1"; }
falha() { echo "FALHA $1"; exit 1; }

# --- gerar.sh: a configuração de cada variante

for v in nvidia padrao; do
  bash "$RAIZ/instalador/gerar.sh" --so-config "$TMP/$v" "$v"
  python3 - "$TMP/$v/config.toml" "$TMP/$v/kickstart.ks" <<'PY'
import sys, tomllib
with open(sys.argv[1], "rb") as f:
    d = tomllib.load(f)
inst = d["customizations"]["installer"]
assert inst["modules"]["disable"] == ["org.fedoraproject.Anaconda.Modules.Users"], inst["modules"]
assert "enable" not in inst["modules"], "nenhum módulo a mais"
assert set(d["customizations"]) == {"installer"}, "só a seção do instalador"
open(sys.argv[2], "w").write(inst["kickstart"]["contents"])
PY
  ok "$v: config.toml lê no TOML e desliga só o módulo Users"

  KS="$TMP/$v/kickstart.ks"
  # O que apagaria disco, escolheria disco, tiraria a tela ou repetiria o que o builder já põe.
  if grep -nEi '^[[:space:]]*(clearpart|autopart|zerombr|ignoredisk|part|partition|reqpart|logvol|volgroup|raid|btrfs|harddrive|cmdline|text|reboot|poweroff|shutdown|halt|ostreecontainer|ostreesetup|user|rootpw)\b|^[[:space:]]*bootc[[:space:]]+--' "$KS"; then
    falha "$v: o kickstart tem diretiva proibida (linha acima)"
  fi
  ok "$v: o kickstart não escolhe disco nem apaga nada"

  grep -qx "bootc switch --mutate-in-place --enforce-container-sigpolicy --transport registry ghcr.io/novasce/novasce-os:stable-$v" "$KS" \
    || falha "$v: falta o bootc switch com a assinatura obrigatória"
  ok "$v: aponta para ghcr.io/novasce/novasce-os:stable-$v com a assinatura obrigatória"

  grep -q "printf 'universalblue\\\\nuniversalblue\\\\n' | mokutil --import \"\$CHAVE_MOK\"" "$KS" \
    || falha "$v: o MOK não está como no ISO do Aurora"
  [ "$(grep -c '^%post' "$KS")" = "$(grep -c '^%end' "$KS")" ] || falha "$v: %post sem %end"
  ok "$v: MOK da Universal Blue como no Aurora"

  if command -v ksvalidator >/dev/null; then
    # Como o builder monta: a cópia da imagem e o switch dele vêm antes do nosso.
    {
      echo "ostreecontainer --url=/run/install/repo/container --transport=oci --no-signature-verification"
      printf '%%post --erroronfail\nbootc switch --mutate-in-place --transport registry ghcr.io/novasce/novasce-os:stable-%s\n%%end\n' "$v"
      cat "$KS"
    } >"$TMP/$v/completo.ks"
    ksvalidator "$TMP/$v/completo.ks" || falha "$v: ksvalidator recusou o kickstart"
    ok "$v: ksvalidator aceita o kickstart montado"
  else
    echo "pula  $v: ksvalidator ausente"
  fi

  P="$TMP/$v/politica.json"
  jq -e '.default == [{"type": "reject"}]' "$P" >/dev/null || falha "política: o padrão não recusa"
  jq -e '.transports.docker["ghcr.io/novasce/novasce-os"] == [{"type": "sigstoreSigned", "keyPath": "/usr/lib/pki/containers/novasce-os.pub", "signedIdentity": {"type": "matchRepository"}}]' "$P" >/dev/null \
    || falha "política: a imagem da casa não exige a assinatura"
  jq -e '.transports.docker | keys == ["ghcr.io/novasce/novasce-os", "quay.io/centos-bootc/bootc-image-builder"]' "$P" >/dev/null \
    || falha "política: aceita mais do que a imagem da casa e o builder"
  ok "$v: a política do pull só aceita a imagem da casa assinada"
done

if bash "$RAIZ/instalador/gerar.sh" --so-config "$TMP/x" outra 2>/dev/null; then
  falha "variante desconhecida passou"
fi
ok "variante desconhecida é recusada"

# --- gravar-pendrive.sh: quem pode ser apagado

# shellcheck disable=SC1091
source "$RAIZ/instalador/gravar-pendrive.sh"
set +e

# Um notebook com o disco do sistema cifrado, um disco USB com o sistema montado,
# o Kingston de 62 GB montado pelo KDE e um pendrive pequeno.
lsblk() {
  cat <<'JSON'
{"blockdevices": [
  {"name": "nvme0n1", "size": 512110190592, "type": "disk", "tran": "nvme", "rm": false, "model": "Samsung", "mountpoints": [null],
   "children": [
     {"name": "nvme0n1p1", "size": 629145600, "type": "part", "tran": null, "rm": false, "model": null, "mountpoints": ["/boot/efi"]},
     {"name": "nvme0n1p3", "size": 510000000000, "type": "part", "tran": null, "rm": false, "model": null, "mountpoints": [null],
      "children": [{"name": "luks-abc", "size": 509000000000, "type": "crypt", "tran": null, "rm": false, "model": null, "mountpoints": ["/sysroot", "/var"]}]}
   ]},
  {"name": "sda", "size": 1000204886016, "type": "disk", "tran": "usb", "rm": false, "model": "Externo", "mountpoints": [null],
   "children": [{"name": "sda1", "size": 1000204000000, "type": "part", "tran": null, "rm": false, "model": null, "mountpoints": ["/var/home"]}]},
  {"name": "sdb", "size": 62008590336, "type": "disk", "tran": "usb", "rm": true, "model": "DataTraveler 3.0", "mountpoints": [null],
   "children": [{"name": "sdb1", "size": 7870000000, "type": "part", "tran": null, "rm": true, "model": null, "mountpoints": ["/run/media/eduardo/Aurora"]}]},
  {"name": "sdc", "size": 8004304896, "type": "disk", "tran": "usb", "rm": true, "model": "Pequeno", "mountpoints": [null]},
  {"name": "sr0", "size": 1073741312, "type": "rom", "tran": "sata", "rm": true, "model": "DVD", "mountpoints": [null]}
]}
JSON
}

ISO=8500000000
recusa() {
  local nome=$1 motivo=$2
  if validar_aparelho "$nome" "$ISO" 2>/dev/null; then falha "aceitou $nome ($motivo)"; fi
  ok "recusa $nome: $motivo"
}
recusa nvme0n1 "disco do sistema, cifrado"
recusa sda "disco USB com o sistema montado"
recusa sdb1 "partição, não o aparelho"
recusa sdc "pequeno demais para o ISO"
recusa sr0 "não é disco"
recusa sdz "não existe"
recusa "" "vazio"
recusa "sdb; rm" "nome com outros caracteres"
validar_aparelho sdb "$ISO" || falha "recusou o Kingston"
ok "aceita sdb: o Kingston no USB, montado só em /run/media"

perguntar() { printf '%s' "$RESPOSTA"; }
RESPOSTA="APAGAR sdb"; confirmar sdb 2>/dev/null || falha "frase certa recusada"
ok "confirma com a frase exata"
for RESPOSTA in "apagar sdb" "APAGAR sdc" "sim" ""; do
  if confirmar sdb 2>/dev/null; then falha "aceitou a frase '$RESPOSTA'"; fi
done
ok "recusa frase diferente, outro aparelho e sim"
