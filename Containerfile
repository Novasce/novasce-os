# Novasce OS · o sistema da máquina da sede.
# Base: Aurora (KDE, Universal Blue) com o driver NVIDIA aberto, que cobre a
# RTX 2060 (Turing) e já vem assinado para o Secure Boot. Nada de driver é
# montado aqui: o kernel e os módulos são os do Aurora.
ARG BASE=ghcr.io/ublue-os/aurora-nvidia-open:stable

FROM scratch AS ctx
COPY build.sh /build.sh
COPY marca /marca

FROM ${BASE}
COPY system_files/ /
RUN --mount=type=bind,from=ctx,source=/,target=/ctx \
    bash /ctx/build.sh && \
    ostree container commit
RUN bootc container lint
