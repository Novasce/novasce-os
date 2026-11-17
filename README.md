# Novasce OS

## TLDR

- O sistema Linux da Novasce, para a máquina da sede e para os notebooks do time, inclusive para quem nunca usou Linux.
- É o Aurora (KDE, Fedora, Universal Blue) montado como imagem bootc, com a marca, os papéis (sede e notebook), a conta Sede com a Sala, os agentes e os apps da casa.
- Duas imagens da mesma receita: `ghcr.io/novasce/novasce-os:stable-nvidia` (sede) e `:stable-padrao` (notebooks), mais `AAAA.MM.DD.N-<variante>` por versão. Montadas no GitHub Actions, assinadas com cosign.
- A máquina só aplica imagem assinada pela casa, atualiza uma vez por semana e nunca reinicia sozinha. Para voltar: `sudo bootc rollback`.
- Repositório e imagem são públicos: nenhum segredo e nenhum conteúdo privado entram aqui.

## O que tem aqui

| Pasta ou arquivo | Para quê |
|---|---|
| `Containerfile` | A receita: estágio `apps` (Rust e Tauri sobre a mesma base) e estágio final |
| `build_files/` | Um script por assunto, rodados em ordem na montagem |
| `system_files/` | Copiado para `/`: unidades, polkit, marca, KDE, Sala, arquivos da sede |
| `marca/` | Cata-vento oficial, papel de parede teal aprovado e as peças do boot |
| `rust/` | Workspace dos serviços: `novasce-core` (contrato com o Tenancy), `novasce-sinal`, `novasce-helper` (root pelo pkexec), `novasce-barra`, `novasce-sede` e os outros (lista no [AGENTS.md](AGENTS.md)) |
| `ui/` | Tokens da marca e componentes React dos apps da casa |
| `apps/` | Os apps da casa em Tauri 2: Preparar esta máquina, Começar, Ajuda, Voz e Sede |
| `ajuda/` | Fonte única da ajuda e do guia |
| `instalador/` | O ISO gerado na própria máquina, a partir das imagens publicadas |
| `notas/` | Notas de versão públicas, lidas pelo aviso de atualização |
| `tests/` | Conferências da imagem montada (`imagem.sh`, `imagem-m3.sh`, no CI), do instalador e da geometria das telas |
| `docs/roteiro-sede.md` | O teste na sede, passo a passo |

## Instalar

O passo a passo, com o resultado esperado em cada passo, está em [docs/roteiro-sede.md](docs/roteiro-sede.md). Em resumo: pendrive com o Aurora, `sudo bootc switch --enforce-container-sigpolicy ghcr.io/novasce/novasce-os:stable-nvidia` (ou `stable-padrao`), reiniciar e seguir o Preparar esta máquina.

## Dia a dia

| Quero | Comando |
|---|---|
| Ver o que roda e o que entra no próximo boot | `rpm-ostree status` |
| Buscar a versão nova agora | `sudo systemctl start uupd.service` |
| Voltar para a versão anterior | `sudo bootc rollback` e reiniciar |
| Registrar a máquina no Tenancy pelo terminal | `sudo novasce-sinal pair` |
| Mandar o sinal agora | `sudo systemctl start novasce-sinal` |
| Log do sinal | `journalctl -u novasce-sinal` |
| Preparar os agentes (só na sede) | `sudo novasce-primeiro-uso` |
| Log dos agentes | `journalctl -u novasce-assistente` e `state/` do executor |

## Desenvolver

O que é preciso para mexer no código (o portão local, o CI e a publicação, as armadilhas e as convenções) está no [AGENTS.md](AGENTS.md). Em resumo:

- Rust: `cd rust && cargo test -p <crate> --locked`.
- Web: `npm ci` na raiz e `npm test -w <pasta>` (`ui`, `ajuda` ou um dos `apps/`).
- A imagem só é montada no CI (`.github/workflows/build.yml`). O `checks.yml` roda shellcheck, clippy, os testes e os apps Tauri.

## Marcos

- Na `main` desde 26/10/2026: a imagem, o sinal e o Preparar (M1); o Começar, a Ajuda, o Sincronizar, a Voz, a volta à Sala, o aviso de atualização, os quatro sinais da barra (falar, nuvem, senhas, ajuda) e o Começar no dock (M3).
- No Tenancy desde 25/10/2026: a Sala, as Telas e as Máquinas (M2).
