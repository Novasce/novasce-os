#!/usr/bin/bash
# Confere, dentro da imagem montada, o que o M3 entrega.
# Uso no CI (workflow build, igual ao tests/imagem.sh do M1):
#   podman run --rm --entrypoint /usr/bin/bash -v "$PWD:/repo:ro" novasce-os:nvidia /repo/tests/imagem-m3.sh
set -uo pipefail
falhou=0
confere() {
  if bash -c "$2" >/dev/null 2>&1; then echo "ok    $1"; else echo "FALHA $1"; falhou=1; fi
}

for b in novasce-comecar novasce-ajuda novasce-voz novasce-sede novasce-sincronizar novasce-barra novasce-tranca novasce-navegador-sede novasce-voz-medir claude codex gemini; do
  confere "bin $b" "test -x /usr/bin/$b"
done
for b in novasce-sinal-local novasce-login-qr git-credencial whisper-cli cli-primeiro-uso novasce-sede-guarda; do
  confere "libexec $b" "test -x /usr/libexec/novasce/$b"
done
confere "modelos da voz" "test -s /usr/share/novasce/voz/ggml-small-q5_1.bin && test -s /usr/share/novasce/voz/ggml-base-q5_1.bin"
confere "vocabulário da voz" "test -s /usr/share/novasce/voz/vocabulario.toml"
confere "apps.toml" "test -s /usr/share/novasce/apps.toml"
confere "índice da ajuda" "test \$(jq length /usr/share/novasce/ajuda/indice.json) -gt 30"
confere "notas de versão" "jq -e 'type == \"array\"' /usr/share/novasce/notas/versoes.json"
confere "socket do sinal local" "systemctl is-enabled novasce-sinal-local.socket | grep -qx enabled"
confere "QR do login" "systemctl is-enabled novasce-login-qr.path | grep -qx enabled"
confere "guarda da Sede ligada" "systemctl is-enabled novasce-sede-guarda.service | grep -qx enabled"
confere "Sede entra sozinha sem Relogin (que quebrava o Trocar usuário)" "grep -qx 'User=sede' /usr/share/novasce/sede/50-novasce-sede-autologin.conf && ! grep -qi '^Relogin' /usr/share/novasce/sede/50-novasce-sede-autologin.conf"
for u in novasce-barra novasce-sincronizar app-org.novasce.voz novasce-ydotoold novasce-sede novasce-tranca; do
  confere "unidade de usuário $u" "systemctl --global is-enabled $u.service | grep -qx enabled"
done
confere "regra uinput" "test -s /usr/lib/udev/rules.d/70-novasce-uinput.rules"
confere "polkit da sede" "test -s /usr/share/polkit-1/rules.d/50-novasce-sede.rules"
confere "executor do KRunner" "test -s /usr/share/krunner/dbusplugins/novasce-busca.desktop"
confere "Começar no login" "grep -q -- '--se-pendente' /etc/xdg/autostart/novasce-comecar.desktop"
confere "ferramentas" "command -v gitleaks rclone ydotool ydotoold wl-copy wl-paste parecord spectacle rsvg-convert"

tmp=$(mktemp -d)
git -C "$tmp" init -q && git -C "$tmp" config user.email t@t && git -C "$tmp" config user.name t
# Token falso gerado na hora: um repetido ("a1B2" x 9) tem entropia baixa e a regra github-pat do
# gitleaks não dispara. Fora do arquivo, para a proteção de push do GitHub não confundir com um real.
falso=$(python3 -c "import secrets, string; print(''.join(secrets.choice(string.ascii_letters + string.digits) for _ in range(36)))")
printf 'token = "ghp_%s"\n' "$falso" >"$tmp/nota.md"
git -C "$tmp" add -A
confere "gitleaks segura token preparado" "gitleaks git --pre-commit --staged --no-banner --exit-code 0 --report-format json --report-path $tmp/r.json $tmp && grep -q nota.md $tmp/r.json"

python3 -c "import wave; w=wave.open('/tmp/silencio.wav','wb'); w.setnchannels(1); w.setsampwidth(2); w.setframerate(16000); w.writeframes(b'\0\0'*16000); w.close()"
confere "whisper-cli carrega o modelo" "/usr/libexec/novasce/whisper-cli -m /usr/share/novasce/voz/ggml-base-q5_1.bin -f /tmp/silencio.wav -l pt -nt -np"
confere "sem travessão nos textos da casa" "! grep -rl \$'\\xe2\\x80\\x94' /usr/share/novasce/apps.toml /usr/share/novasce/ajuda /usr/share/applications/novasce-ajuda.desktop /usr/share/applications/novasce-comecar.desktop"
exit "$falhou"
