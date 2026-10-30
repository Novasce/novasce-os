#!/usr/bin/env bash
# Novasce OS · grava o instalador da casa num pendrive, no Linux.
#
#   sudo bash instalador/gravar-pendrive.sh [arquivo.iso]
#
# Sem o arquivo, lista os ISOs de /var/lib/novasce-instalador e pergunta qual.
# Nunca escolhe o aparelho sozinho: mostra o lsblk, pede o nome e depois a frase
# "APAGAR <nome>". Recusa o disco do sistema e o que não estiver no USB.
# Depois de gravar, lê o pendrive de volta e confere com o sha256 do ISO.
set -euo pipefail

SAIDA=${NOVASCE_SAIDA:-/var/lib/novasce-instalador}

perguntar() {
  local texto=$1 resposta
  read -r -p "$texto" resposta </dev/tty
  printf '%s' "$resposta"
}

escolher_iso() {
  local isos=() i
  shopt -s nullglob
  isos=("$SAIDA"/*.iso)
  shopt -u nullglob
  [ "${#isos[@]}" -gt 0 ] || { echo "Nenhum ISO em $SAIDA. Rode antes o gerar.sh." >&2; return 1; }
  for i in "${!isos[@]}"; do
    printf '  %d) %s (%s)\n' "$((i + 1))" "$(basename "${isos[$i]}")" "$(du -h "${isos[$i]}" | cut -f1)" >&2
  done
  local n
  n=$(perguntar "Qual ISO gravar (número): ")
  if ! [[ "$n" =~ ^[0-9]+$ ]] || [ "$n" -lt 1 ] || [ "$n" -gt "${#isos[@]}" ]; then
    echo "Número fora da lista: '$n'." >&2
    return 1
  fi
  printf '%s' "${isos[$((n - 1))]}"
}

# Confere se o nome digitado é um pendrive que pode ser apagado.
# Lê o lsblk em JSON; recebe o nome (sdb) e o tamanho do ISO em bytes.
validar_aparelho() {
  local nome=$1 bytes_iso=$2 json disco
  [[ "$nome" =~ ^[a-z][a-z0-9]*$ ]] || { echo "Nome inválido: '$nome'. Digite só o nome do aparelho, por exemplo sdb." >&2; return 1; }
  json=$(lsblk -J -b -o NAME,SIZE,TYPE,TRAN,RM,MODEL,MOUNTPOINTS)
  disco=$(jq -c --arg n "$nome" '.blockdevices[] | select(.name == $n)' <<<"$json")
  [ -n "$disco" ] || { echo "Não existe o aparelho '$nome' no lsblk." >&2; return 1; }
  [ "$(jq -r .type <<<"$disco")" = disk ] \
    || { echo "'$nome' não é um disco inteiro. Digite o aparelho (sdb), não a partição (sdb1)." >&2; return 1; }
  [ "$(jq -r '.tran // ""' <<<"$disco")" = usb ] \
    || { echo "'$nome' não está no USB. Este script só grava pendrive." >&2; return 1; }
  # Disco do sistema ou cifrado: qualquer ponto de montagem fora de /run/media, ou volume cifrado/LVM dentro.
  local sistema
  sistema=$(jq -r '[.. | objects | select(has("name"))
      | select((.type == "crypt") or (.type == "lvm")
               or ((.mountpoints // []) | map(select(. != null and (startswith("/run/media/") | not))) | length > 0))
      | .name] | join(" ")' <<<"$disco")
  [ -z "$sistema" ] \
    || { echo "'$nome' está em uso pelo sistema ($sistema). Não é o pendrive." >&2; return 1; }
  local tamanho
  tamanho=$(jq -r .size <<<"$disco")
  [ "$tamanho" -gt "$bytes_iso" ] \
    || { echo "'$nome' tem $((tamanho / 1000000000)) GB; o ISO não cabe." >&2; return 1; }
  return 0
}

descrever() {
  local nome=$1
  lsblk -dno SIZE,TRAN,MODEL "/dev/$nome" | sed 's/  */ /g'
}

confirmar() {
  local nome=$1 frase
  frase=$(perguntar "Para apagar tudo em /dev/$nome, digite APAGAR $nome: ")
  [ "$frase" = "APAGAR $nome" ] || { echo "Frase diferente. Nada foi gravado." >&2; return 1; }
}

gravar() {
  local iso=$1 nome=$2 bytes=$3
  local parte
  for parte in $(lsblk -lno NAME,MOUNTPOINTS "/dev/$nome" | awk 'NF > 1 {print $1}'); do
    echo "Desmontando /dev/$parte"
    umount "/dev/$parte"
  done
  echo "== gravando $(basename "$iso") em /dev/$nome ($(date +%T))"
  dd if="$iso" of="/dev/$nome" bs=4M oflag=direct conv=fsync status=progress
  sync
  # Lê do pendrive, não do cache.
  blockdev --flushbufs "/dev/$nome"
  echo "== conferindo a gravação ($(date +%T))"
  local esperado lido
  esperado=$(cut -d' ' -f1 "$iso.sha256")
  lido=$(head -c "$bytes" "/dev/$nome" | sha256sum | cut -d' ' -f1)
  if [ "$esperado" = "$lido" ]; then
    echo "Gravação conferida: o pendrive bate com o ISO ($esperado)."
  else
    echo "!! O pendrive NÃO bate com o ISO (esperado $esperado, lido $lido). Grave de novo." >&2
    return 1
  fi
}

main() {
  [ "$(id -u)" -eq 0 ] || { echo "Rode como root: sudo bash $0" >&2; exit 1; }
  local iso=${1:-}
  [ -n "$iso" ] || iso=$(escolher_iso)
  [ -f "$iso" ] || { echo "Não achei o ISO: $iso" >&2; exit 1; }
  [ -f "$iso.sha256" ] || { echo "Falta $iso.sha256 ao lado do ISO." >&2; exit 1; }
  echo "== conferindo o ISO"
  (cd "$(dirname "$iso")" && sha256sum -c "$(basename "$iso").sha256")
  local bytes
  bytes=$(stat -c %s "$iso")

  echo
  echo "== aparelhos (o pendrive é o que tem TRAN usb e RM 1)"
  lsblk -e7 -o NAME,SIZE,TYPE,TRAN,RM,MODEL,MOUNTPOINTS
  echo
  local nome
  nome=$(perguntar "Nome do pendrive na coluna NAME (por exemplo sdb): ")
  validar_aparelho "$nome" "$bytes" || exit 1
  echo "Vai APAGAR tudo em /dev/$nome: $(descrever "$nome")"
  confirmar "$nome" || exit 1
  gravar "$iso" "$nome" "$bytes"
  echo "Pronto. Pode tirar o pendrive."
}

if [ "${BASH_SOURCE[0]}" = "$0" ]; then
  main "$@"
fi
