# Ícones dos apps da casa

O dock do Novasce OS mostra a marca de cada app, nunca uma letra num quadrado. Logo não se
gera nem se redesenha: o `compor.py` pega o arquivo oficial de `fontes/`, recorta no que tem
tinta, centra e grava o PNG de 256 px em `system_files/usr/share/icons/hicolor/256x256/apps/`.

| App | Fonte | De onde veio |
|---|---|---|
| Tenancy | `fontes/tenancy.png` | o ícone do próprio Tenancy web (`novasce-tenancy-app/src/app/icon.png`, commit `ebbbb3a`), reduzido para 512 px |
| Claude | `fontes/claude.png` | o ícone que claude.ai publica, baixado em 26/10/2026 |
| ChatGPT | `fontes/chatgpt.png` | o ícone que chatgpt.com publica, baixado em 26/10/2026 |
| Gemini | `fontes/gemini.png` | o ícone que gemini.google.com publica, baixado em 26/10/2026 |
| WhatsApp | `fontes/whatsapp.png` | o ícone que web.whatsapp.com publica, baixado em 26/10/2026 |

Os quatro de fora vieram pelo serviço de ícones do Google (`t2.gstatic.com/faviconV2`), que
entrega o ícone que cada site declara. Os sites recusam o acesso direto sem navegador.

O Material da casa não tem marca própria: usa o `folder-documents` do tema Breeze.

Para refazer depois de trocar uma fonte: `python3 marca/apps/compor.py` (precisa do Pillow).
