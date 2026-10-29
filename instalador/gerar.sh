#!/usr/bin/env bash
# Novasce OS · gera o instalador da casa (pendrive que já instala o Novasce OS).
#
# Roda na sede, como root, a partir das imagens assinadas do GHCR:
#   sudo bash instalador/gerar.sh [as-duas|nvidia|padrao]
# Sai em /var/lib/novasce-instalador/: o ISO, o .sha256 e um .txt com a versão e o digest.
#
# Só escreve a configuração do builder, sem root e sem podman (é o que o teste usa):
#   bash instalador/gerar.sh --so-config <pasta> <nvidia|padrao>
#
# O que o ISO faz (anaconda-iso do bootc-image-builder, conferido no código do osbuild/images):
# - Sem kickstart nosso, o builder gera um kickstart automático que apaga todos os discos
#   (clearpart --all e autopart), sem cifra. Por isso o kickstart da casa vai sempre, e ele
#   não escolhe disco nem apaga nada: a escolha do disco e a cifra ficam na tela do instalador.
# - O builder já põe a cópia da imagem e um "bootc switch" sem conferir assinatura. O nosso
#   %post vem depois dele e troca para o GHCR com a assinatura obrigatória.
# - A conta não nasce no instalador: o módulo Users fica desligado e a conta nasce no primeiro
#   boot, no Plasma Setup, como no Aurora. O MOK da Universal Blue entra como no ISO do Aurora.
set -euo pipefail

IMAGEM=ghcr.io/novasce/novasce-os
BIB=${NOVASCE_BIB:-quay.io/centos-bootc/bootc-image-builder:latest}
SAIDA=${NOVASCE_SAIDA:-/var/lib/novasce-instalador}
CHAVE=/usr/lib/pki/containers/novasce-os.pub
ANEXOS=/etc/containers/registries.d/novasce-os.yaml
LIVRE_MIN_GB=45

variantes() {
  case "${1:-as-duas}" in
    as-duas) echo "nvidia padrao" ;;
    nvidia | padrao) echo "$1" ;;
    *)
      echo "Variante desconhecida: '$1'. Use nvidia, padrao ou as-duas." >&2
      return 1
      ;;
  esac
}

# config.toml do bootc-image-builder para uma variante.
escrever_config() {
  local variante=$1 pasta=$2
  local ref="${IMAGEM}:stable-${variante}"
  mkdir -p "$pasta"
  cat >"$pasta/config.toml" <<EOF
# Novasce OS · instalador ${variante} (gerado por instalador/gerar.sh).

[customizations.installer.modules]
# A conta nasce no primeiro boot, no Plasma Setup, como no Aurora.
disable = ["org.fedoraproject.Anaconda.Modules.Users"]

[customizations.installer.kickstart]
# String literal ('''): o TOML não mexe nas barras do kickstart.
contents = '''
# Disco, cifra, idioma e rede ficam com quem instala. Este arquivo nao escolhe disco
# nem apaga nada: sem clearpart, autopart, part, zerombr nem ignoredisk.

%post --erroronfail
set -e
# A maquina passa a buscar a imagem no GHCR, e a primeira atualizacao ja confere a assinatura da casa.
bootc switch --mutate-in-place --enforce-container-sigpolicy --transport registry ${ref}
%end

%post
# A chave da Universal Blue (o driver NVIDIA e assinado por ela), como no ISO do Aurora:
# a tela azul do MOK aparece no primeiro boot e pede a senha universalblue.
CHAVE_MOK=/etc/pki/akmods/certs/akmods-ublue.der
if [ ! -d /sys/firmware/efi ]; then echo "Sem EFI: o MOK fica de fora."; exit 0; fi
if [ ! -f "\$CHAVE_MOK" ]; then echo "Sem \$CHAVE_MOK na imagem: o MOK fica de fora."; exit 0; fi
mokutil --timeout -1 || :
printf 'universalblue\nuniversalblue\n' | mokutil --import "\$CHAVE_MOK" || :
%end
'''
EOF
}

# Política só para este pull: a imagem da casa só entra assinada com a chave da casa.
escrever_politica() {
  local pasta=$1
  mkdir -p "$pasta"
  jq -n --arg chave "$CHAVE" --arg imagem "$IMAGEM" --arg bib "${BIB%%[:@]*}" '{
    default: [{type: "reject"}],
    transports: {
      docker: {
        ($imagem): [{type: "sigstoreSigned", keyPath: $chave, signedIdentity: {type: "matchRepository"}}],
        ($bib): [{type: "insecureAcceptAnything"}]
      }
    }
  }' >"$pasta/politica.json"
}

conferir_maquina() {
  [ "$(id -u)" -eq 0 ] || { echo "Rode como root: sudo bash $0" >&2; exit 1; }
  [ "$(uname -m)" = x86_64 ] || { echo "O instalador é x86_64; esta máquina é $(uname -m)." >&2; exit 1; }
  local falta=""
  for c in podman jq sha256sum df; do command -v "$c" >/dev/null || falta="$falta $c"; done
  [ -z "$falta" ] || { echo "Faltam:$falta" >&2; exit 1; }
  [ -s "$CHAVE" ] || { echo "Falta a chave da casa em $CHAVE. Rode numa máquina com o Novasce OS." >&2; exit 1; }
  grep -q 'use-sigstore-attachments: true' "$ANEXOS" 2>/dev/null \
    || { echo "Falta $ANEXOS com use-sigstore-attachments: sem ele a assinatura não é lida." >&2; exit 1; }
  mkdir -p "$SAIDA"
  local livre
  livre=$(df -BG --output=avail "$SAIDA" | tail -n 1 | tr -dc 0-9)
  local raiz
  raiz=$(df -BG --output=avail /var/lib/containers 2>/dev/null | tail -n 1 | tr -dc 0-9)
  echo "Espaço livre: ${livre} GB em $SAIDA, ${raiz:-?} GB em /var/lib/containers."
  [ "$livre" -ge "$LIVRE_MIN_GB" ] || { echo "Precisa de ${LIVRE_MIN_GB} GB livres." >&2; exit 1; }
}

baixar() {
  local ref=$1 politica=$2
  echo "== baixando $ref"
  podman pull --signature-policy "$politica" "$ref"
}

gerar_iso() {
  local variante=$1 trabalho=$2
  local ref="${IMAGEM}:stable-${variante}"
  local pasta="$trabalho/$variante"
  escrever_config "$variante" "$pasta"
  baixar "$ref" "$trabalho/politica.json"

  local digest versao
  digest=$(podman image inspect --format '{{.Digest}}' "$ref")
  versao=$(podman image inspect --format '{{index .Labels "org.opencontainers.image.version"}}' "$ref")
  echo "== $variante: versão $versao, digest $digest"

  mkdir -p "$pasta/saida" "$SAIDA/cache" "$SAIDA/rpmmd"
  echo "== gerando o ISO $variante ($(date +%T))"
  podman run --rm --privileged --pull=never \
    --security-opt label=type:unconfined_t \
    -v "$pasta/config.toml:/config.toml:ro" \
    -v "$pasta/saida:/output" \
    -v "$SAIDA/cache:/store" \
    -v "$SAIDA/rpmmd:/rpmmd" \
    -v /var/lib/containers/storage:/var/lib/containers/storage \
    "$BIB" \
    --type anaconda-iso \
    --rootfs btrfs \
    "$ref"
  echo "== ISO $variante pronto ($(date +%T))"

  local iso="$SAIDA/novasce-os-${versao}-${variante}.iso"
  mv "$pasta/saida/bootiso/install.iso" "$iso"
  (cd "$SAIDA" && sha256sum "$(basename "$iso")" >"$(basename "$iso").sha256")
  {
    echo "variante=$variante"
    echo "versao=$versao"
    echo "imagem=$ref"
    echo "digest=$digest"
    echo "builder=$BIB"
    echo "builder_digest=$(podman image inspect --format '{{.Digest}}' "$BIB")"
    echo "gerado=$(date -Iseconds)"
    echo "maquina=$(hostname)"
  } >"${iso%.iso}.txt"
  chmod 0644 "$iso" "$iso.sha256" "${iso%.iso}.txt"
  echo "== $iso"
  ls -lh "$iso"
  cat "$iso.sha256" "${iso%.iso}.txt"
}

main() {
  if [ "${1:-}" = --so-config ]; then
    local pasta=${2:?pasta}
    variantes "${3:?variante}" >/dev/null
    escrever_config "$3" "$pasta"
    escrever_politica "$pasta"
    return 0
  fi

  local lista
  lista=$(variantes "${1:-as-duas}")
  conferir_maquina

  local trabalho
  trabalho=$(mktemp -d "$SAIDA/trabalho-XXXXXX")
  trap 'rm -rf "$trabalho"' EXIT
  escrever_politica "$trabalho"
  baixar "$BIB" "$trabalho/politica.json"

  local v
  for v in $lista; do
    gerar_iso "$v" "$trabalho"
  done
  echo
  echo "Pronto. Os instaladores estão em $SAIDA:"
  ls -lh "$SAIDA"/*.iso
  echo "Para gravar o pendrive: sudo bash instalador/gravar-pendrive.sh"
}

if [ "${BASH_SOURCE[0]}" = "$0" ]; then
  main "$@"
fi
