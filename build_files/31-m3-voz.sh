#!/usr/bin/bash
# Modelos do whisper.cpp (públicos, MIT), conferidos pela soma registrada no repositório.
set -euxo pipefail
DEST=/usr/share/novasce/voz
mkdir -p "$DEST"
BASE=https://huggingface.co/ggerganov/whisper.cpp/resolve/main
for m in ggml-small-q5_1.bin ggml-base-q5_1.bin; do
  curl -fsSL --retry 3 -o "$DEST/$m" "$BASE/$m"
done
(cd "$DEST" && sha256sum -c /ctx/build_files/voz-modelos.sha256)
test -x /usr/libexec/novasce/whisper-cli
