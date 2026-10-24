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
| `rust/` | `novasce-core` (contrato com o Tenancy), `novasce-sinal` (serviço) e `novasce-helper` (root pelo pkexec) |
| `ui/` | Tokens da marca e componentes React dos apps da casa |
| `apps/preparar/` | Preparar esta máquina (Tauri 2) |
| `notas/` | Notas de versão públicas, lidas pelo aviso de atualização |
| `tests/imagem.sh` | Confere a imagem montada, no CI |
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

- Rust: `cd rust && cargo test -p <crate>`.
- Web: `npm install` na raiz e `npm test -w ui` ou `npm test -w apps/preparar`.
- A imagem só é montada no CI (`.github/workflows/build.yml`). O CI também roda shellcheck, clippy, os testes e o lado Rust do Preparar.

## Fora do M1

- Começar, Ajuda, Sincronizar, Voz, a volta à Sala e o aviso de atualização: M3.
- Os quatro sinais da barra (falar, nuvem, senhas, ajuda) e o Começar no dock: M3.
- Sala, Telas e Máquinas no Tenancy: M2.
