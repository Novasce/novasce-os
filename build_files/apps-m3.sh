#!/usr/bin/bash
# Roda no estágio "apps" do Containerfile, em /src, depois do Preparar.
# Compila o que o M3 entrega e grava em /out, que vira a raiz da imagem.
set -euxo pipefail
cd /src

dnf5 install -y --setopt=install_weak_deps=False cmake git gtk3-devel gtk-layer-shell-devel dbus-devel

# whisper.cpp na CPU, com AVX2 e sem GPU (a voz confere AVX2 antes de ouvir).
git clone --depth 1 --branch v1.7.6 https://github.com/ggml-org/whisper.cpp /tmp/whisper
cmake -S /tmp/whisper -B /tmp/whisper/build -DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=OFF \
  -DGGML_NATIVE=OFF -DGGML_AVX=ON -DGGML_AVX2=ON -DGGML_FMA=ON -DGGML_F16C=ON -DWHISPER_BUILD_TESTS=OFF
cmake --build /tmp/whisper/build -j"$(nproc)" --target whisper-cli
install -Dm0755 /tmp/whisper/build/bin/whisper-cli /out/usr/libexec/novasce/whisper-cli

# Ajuda: índice para a busca e a IA (a mesma fonte da Central e do Guia).
npm run guia -w ajuda -- --versao v1 --data "$(date +%d/%m/%Y)"
install -Dm0644 ajuda/saida/indice.json /out/usr/share/novasce/ajuda/indice.json

# Binários Rust sem tela.
cargo build --release --locked --manifest-path rust/Cargo.toml \
  -p novasce-sinal-local -p novasce-sincronizar -p novasce-barra -p novasce-sede -p novasce-comecar -p novasce-voz -p novasce-chamados
R=rust/target/release
install -Dm0755 "$R/novasce-sinal-local" /out/usr/libexec/novasce/novasce-sinal-local
install -Dm0755 "$R/novasce-login-qr" /out/usr/libexec/novasce/novasce-login-qr
install -Dm0755 "$R/novasce-git-credencial" /out/usr/libexec/novasce/git-credencial
for b in novasce-sincronizar novasce-barra novasce-tranca novasce-navegador-sede novasce-voz-medir; do
  install -Dm0755 "$R/$b" "/out/usr/bin/$b"
done

# Apps Tauri, como o Preparar: sem pacote, só o binário.
for app in comecar ajuda voz sede; do
  (cd "apps/$app" && npx tauri build --no-bundle -- --locked)
  install -Dm0755 "apps/$app/src-tauri/target/release/novasce-$app" "/out/usr/bin/novasce-$app"
done
