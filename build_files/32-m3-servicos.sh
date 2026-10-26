#!/usr/bin/bash
# Liga os serviços do M3, os atalhos das CLIs de IA, as notas de versão e o ícone "pronta".
set -euxo pipefail

chmod 0755 /usr/libexec/novasce/cli-primeiro-uso
for c in claude codex gemini; do ln -sf /usr/libexec/novasce/cli-primeiro-uso "/usr/bin/$c"; done

install -Dm0644 /ctx/notas/versoes.json /usr/share/novasce/notas/versoes.json

# Cata-vento oficial, branco, sobre o quadrado teal. O vetor não é redesenhado: só é composto.
interno=$(sed '1d;$d' /ctx/marca/catavento.svg)
install -d /usr/share/icons/hicolor/scalable/status
cat >/usr/share/icons/hicolor/scalable/status/novasce-atualizacao.svg <<EOF
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect width="24" height="24" rx="6" fill="#00666C"/><svg x="5" y="5" width="14" height="14" viewBox="0 0 142.582 143.999" fill="#F5F5F7">$interno</svg></svg>
EOF
gtk-update-icon-cache -f /usr/share/icons/hicolor || true

# Cliente OAuth do Google para o Drive: público por definição do Google (app "Desktop"),
# vem das variáveis do Actions, nunca do git. Sem elas, o rclone usa o cliente padrão.
if [ -n "${NOVASCE_GOOGLE_CLIENT_ID:-}" ] && [ -n "${NOVASCE_GOOGLE_CLIENT_SECRET:-}" ]; then
  awk -v id="$NOVASCE_GOOGLE_CLIENT_ID" -v seg="$NOVASCE_GOOGLE_CLIENT_SECRET" '
    /^\[/ { secao = $0 }
    secao == "[google]" && /^client_id = / { print "client_id = \"" id "\""; next }
    secao == "[google]" && /^client_secret = / { print "client_secret = \"" seg "\""; next }
    { print }' /usr/share/novasce/apps.toml >/tmp/apps.toml
  mv /tmp/apps.toml /usr/share/novasce/apps.toml
fi

systemctl enable novasce-sinal-local.socket novasce-login-qr.service novasce-login-qr.path
systemctl --global enable novasce-barra.service novasce-sincronizar.service app-org.novasce.voz.service \
  novasce-ydotoold.service novasce-sede.service novasce-tranca.service
