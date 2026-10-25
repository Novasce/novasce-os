# Novasce OS · a receita da imagem (bootc sobre o Aurora).
# Uma receita, duas imagens: BASE_IMAGE com NVIDIA (sede) ou padrão (notebooks).
# O estágio "apps" compila o Rust e o Tauri SOBRE A MESMA BASE: assim a glibc e
# o webkit2gtk4.1 que o Preparar usa são os mesmos que rodam na máquina.
ARG BASE_IMAGE=ghcr.io/ublue-os/aurora-nvidia-open:stable

FROM scratch AS ctx
COPY build_files /build_files
COPY marca /marca
COPY cosign.pub /cosign.pub

FROM ${BASE_IMAGE} AS apps
RUN --mount=type=cache,dst=/var/cache \
    --mount=type=tmpfs,dst=/tmp \
    dnf5 install -y --setopt=install_weak_deps=False \
      rust cargo gcc gcc-c++ make pkgconf-pkg-config nodejs npm \
      webkit2gtk4.1-devel openssl-devel librsvg2-devel libappindicator-gtk3-devel
# /root aponta para /var/roothome, que não existe na montagem: sem ela, o cargo e o npm não
# criam ~/.cargo e ~/.npm. Só neste estágio; a imagem final leva apenas o /out.
RUN mkdir -p /var/roothome
WORKDIR /src
COPY package.json package-lock.json ./
COPY ui ui
COPY apps apps
COPY rust rust
RUN cargo build --release --locked --manifest-path rust/Cargo.toml -p novasce-sinal -p novasce-helper
RUN npm ci && cd apps/preparar && npx tauri build --no-bundle -- --locked
RUN install -Dm0755 rust/target/release/novasce-sinal /out/usr/bin/novasce-sinal \
 && install -Dm0755 rust/target/release/novasce-helper /out/usr/libexec/novasce/novasce-helper \
 && install -Dm0755 apps/preparar/src-tauri/target/release/novasce-preparar /out/usr/bin/novasce-preparar

FROM ${BASE_IMAGE}
ARG VARIANTE=nvidia
ARG IMAGE_VERSION=0000.00.00.0
# Os caminhos do Aurora que a Novasce substitui. Se mudarem, a montagem para aqui.
RUN --mount=type=bind,from=ctx,source=/,target=/ctx \
    bash /ctx/build_files/conferir-aurora.sh
COPY system_files/ /
COPY --from=apps /out/ /
RUN --mount=type=bind,from=ctx,source=/,target=/ctx \
    --mount=type=cache,dst=/var/cache \
    --mount=type=cache,dst=/var/log \
    --mount=type=tmpfs,dst=/tmp \
    VARIANTE="${VARIANTE}" IMAGE_VERSION="${IMAGE_VERSION}" bash /ctx/build_files/rodar.sh
RUN bootc container lint
