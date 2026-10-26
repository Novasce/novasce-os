#!/usr/bin/bash
# Tela de boot: tema próprio no plugin script. As imagens saem do vetor oficial.
# O initramfs é refeito no 90-initramfs.sh, que leva o tema para o boot.
set -euo pipefail

TEMA=/usr/share/plymouth/themes/novasce
PECAS=/ctx/marca/plymouth

test -f "$TEMA/novasce.plymouth"
test -f "$TEMA/novasce.script"
test -f /usr/lib64/plymouth/script.so
grep -qx 'Theme=novasce' /etc/plymouth/plymouthd.conf

sed 's/fill="currentColor"/fill="#F5F5F7"/' /ctx/marca/catavento.svg |
  rsvg-convert -h 368 --keep-aspect-ratio -o "$TEMA/catavento.png"
for peca in trilho linha ponto nome pedido; do
  rsvg-convert "$PECAS/$peca.svg" -o "$TEMA/$peca.png"
done

# Nada de animação de giro no tema.
shopt -s nullglob
giro=("$TEMA"/throbber* "$TEMA"/animation* "$TEMA"/spinner*)
if [ "${#giro[@]}" -gt 0 ]; then
  echo "O tema de boot não pode ter animação de giro: ${giro[*]}" >&2
  exit 1
fi
