#!/usr/bin/env python3
"""Compõe os ícones dos apps da casa a partir dos arquivos oficiais em fontes/.

Nada é redesenhado: cada marca entra inteira, só recortada no que tem tinta,
centrada e redimensionada. O Tenancy é o ícone do próprio Tenancy web, com os
cantos arredondados; os outros vão sobre um quadrado branco com borda fina.

Uso (precisa do Pillow):  python3 marca/apps/compor.py
Grava em system_files/usr/share/icons/hicolor/256x256/apps/novasce-<app>.png.
"""
import os
import sys

from PIL import Image, ImageChops, ImageDraw

AQUI = os.path.dirname(os.path.abspath(__file__))
DESTINO = os.path.join(AQUI, "..", "..", "system_files/usr/share/icons/hicolor/256x256/apps")
LADO = 256
RAIO = 0.176  # o mesmo canto do ícone do Preparar (180 de 1024)
MARCA = 0.60  # a marca ocupa 60% do lado
BORDA = "#E5E9E8"  # a "linha" da casa
SOBRE_BRANCO = ["claude", "chatgpt", "gemini", "whatsapp"]


def mascara(lado, escala=4):
    """Quadrado de cantos arredondados, desenhado 4x maior para a borda sair lisa."""
    grande = Image.new("L", (lado * escala, lado * escala), 0)
    ImageDraw.Draw(grande).rounded_rectangle(
        (0, 0, lado * escala - 1, lado * escala - 1), radius=int(lado * escala * RAIO), fill=255
    )
    return grande.resize((lado, lado), Image.LANCZOS)


def tinta(imagem):
    """A caixa do que não é fundo: nem transparente, nem branco."""
    rgba = imagem.convert("RGBA")
    r, g, b, a = rgba.split()
    escuro = ImageChops.darker(ImageChops.darker(r, g), b).point(lambda v: 255 if v < 240 else 0)
    opaco = a.point(lambda v: 255 if v > 16 else 0)
    caixa = ImageChops.multiply(escuro, opaco).getbbox()
    if caixa is None:
        sys.exit("arquivo sem marca nenhuma")
    return rgba.crop(caixa)


def sobre_branco(nome):
    marca = tinta(Image.open(os.path.join(AQUI, "fontes", nome + ".png")))
    alvo = int(LADO * MARCA)
    escala = alvo / max(marca.size)
    marca = marca.resize((round(marca.width * escala), round(marca.height * escala)), Image.LANCZOS)
    escala_borda = 4
    grande = Image.new("RGBA", (LADO * escala_borda,) * 2, (0, 0, 0, 0))
    ImageDraw.Draw(grande).rounded_rectangle(
        (0, 0, LADO * escala_borda - 1, LADO * escala_borda - 1),
        radius=int(LADO * escala_borda * RAIO),
        fill="#FFFFFF",
        outline=BORDA,
        width=2 * escala_borda,
    )
    ladrilho = grande.resize((LADO, LADO), Image.LANCZOS)
    ladrilho.alpha_composite(marca, ((LADO - marca.width) // 2, (LADO - marca.height) // 2))
    return ladrilho


def tenancy():
    icone = Image.open(os.path.join(AQUI, "fontes", "tenancy.png")).convert("RGBA")
    icone = icone.resize((LADO, LADO), Image.LANCZOS)
    icone.putalpha(mascara(LADO))
    return icone


def main():
    os.makedirs(DESTINO, exist_ok=True)
    feitos = {"tenancy": tenancy()}
    for nome in SOBRE_BRANCO:
        feitos[nome] = sobre_branco(nome)
    for nome, imagem in feitos.items():
        caminho = os.path.normpath(os.path.join(DESTINO, "novasce-%s.png" % nome))
        imagem.save(caminho, optimize=True)
        print(caminho)


if __name__ == "__main__":
    main()
