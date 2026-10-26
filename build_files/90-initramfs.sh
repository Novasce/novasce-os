#!/usr/bin/bash
# A tela de boot (inclusive o pedido da senha do disco) mora no initramfs, então
# ele é refeito com o tema novo. Mesmos módulos do Aurora: as configurações do
# dracut dele continuam em /usr/lib/dracut/dracut.conf.d.
set -euo pipefail

KERNEL_VERSION=$(rpm -q --queryformat '%{evr}.%{arch}' kernel-core)
INITRAMFS="/usr/lib/modules/${KERNEL_VERSION}/initramfs.img"

DRACUT_NO_XATTR=1 dracut --kver "$KERNEL_VERSION" --reproducible --force --tmpdir /tmp "$INITRAMFS"
chmod 0600 "$INITRAMFS"
lsinitrd "$INITRAMFS" | grep 'usr/share/plymouth/themes/novasce/novasce.script' >/dev/null
