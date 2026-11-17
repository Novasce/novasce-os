# Novasce OS: regras para quem escreve código aqui

Vale para Claude, Codex, agy e para quem escreve à mão. O `CLAUDE.md` só importa este arquivo. O que o sistema é, para quem serve e os comandos do dia a dia da máquina estão no [README](README.md); aqui fica o que é preciso para mexer neste código.

**Repositório e imagem são públicos.** Nenhum segredo, chave privada, token, senha de Wi-Fi, dado de pessoa ou caminho de máquina entra em arquivo, teste, fixture, log ou commit.

## O que é e com quem conversa

- Imagem bootc sobre o Aurora (Fedora, KDE Plasma, Universal Blue). Uma receita (`Containerfile`), duas imagens: `ghcr.io/novasce/novasce-os:stable-nvidia` (base `aurora-nvidia-open:stable`) e `:stable-padrao` (base `aurora:stable`).
- Conversa com a API do Tenancy pelas rotas `/os/*` (`rust/crates/novasce-core/src/tenancy.rs`, tipos em `contrato.rs`; endereços em `system_files/usr/share/novasce/config.toml`), com o GitHub (Device Flow do Começar), o Google Drive (rclone), o Flathub e o GHCR.
- Pastas: `build_files/` (um script por assunto), `system_files/` (copiado para `/`), `marca/`, `rust/` (workspace dos serviços), `apps/` (cinco apps Tauri 2: `preparar`, `comecar`, `ajuda`, `voz`, `sede`), `ui/` (tokens e componentes React), `ajuda/` (fonte única da ajuda e do guia), `notas/`, `instalador/` (o ISO gerado na própria máquina), `tests/`.
- Crates de `rust/crates/`: `novasce-core` (config, contrato, token, leituras do sistema), `novasce-sinal` (sinal diário), `novasce-helper` (root pelo pkexec), `novasce-sinal-local` (root por socket), `novasce-casa` (peças comuns), `novasce-chamados`, `novasce-comecar`, `novasce-sincronizar`, `novasce-voz`, `novasce-camada` (janela em layer-shell), `novasce-barra`, `novasce-sede`.
- `docs/roteiro-sede.md` é o teste na máquina, passo a passo. `docs/m3-interface-m1.md` é retrato de 23/10/2026, não estado.

## O portão local: o PR se prova aqui e no CI

1. Shell (roda no macOS e no Linux, em segundos): `shellcheck build_files/*.sh tests/*.sh .github/scripts/*.sh instalador/*.sh system_files/usr/libexec/novasce/* system_files/usr/bin/novasce-primeiro-uso`, `bash tests/versao.test.sh` e `bash tests/instalador.test.sh`. Sem o `ksvalidator`, o teste do instalador pula a validação do kickstart; para ela, um venv com `pykickstart==3.78`, como no `checks.yml`.
2. Rust: `cd rust && cargo fmt --all -- --check`, depois `cargo test -p <crate> --locked` e `cargo clippy -p <crate> --all-targets --locked -- -D warnings` do que mudou. No macOS, o que é só do Linux (`ksni`, `zbus`, `keyring`, Wayland, `nix`, GTK) fica fora pelo `cfg(target_os = "linux")`: a lógica compila e testa, o resto só o CI confere.
3. Web: `npm ci` na raiz uma vez; depois `npm run typecheck -w <pasta>` e `npm test -w <pasta>` (`ui`, `ajuda`, `apps/preparar`, `apps/comecar`, `apps/ajuda`, `apps/voz`, `apps/sede`). Sem `-w`, roda todos.
4. Texto de tela: `npm run guia -w ajuda -- --versao v1 --data 01/01/2026` e o grep do `checks.yml`, que não pode achar nada: `grep -rn $'\xe2\x80\x94' apps/*/src ajuda/conteudo ajuda/guia system_files/usr/share/novasce`.
5. Mexeu em tela do Começar ou da Ajuda: `cd tests/geometria && npm install && npx playwright install chromium && npx playwright test` (sobe os dois apps com `VITE_PONTE=falsa`). Pesa: rode sozinho.
6. Mexeu em `notas/versoes.json`: `cd rust && cargo test -p novasce-core` lê o arquivo do repositório.

- **Só no CI (Linux x86_64):** os apps Tauri (`npm run build -w apps/<app>` e `cargo test --locked --manifest-path apps/<app>/src-tauri/Cargo.toml`, com as bibliotecas listadas no `checks.yml`), o clippy com o código do Linux, a montagem das duas imagens e as conferências `tests/imagem.sh` e `tests/imagem-m3.sh`, que rodam dentro da imagem montada.
- **Só na máquina:** boot, login, Sala, TPM, atualização e volta de versão. O roteiro marca com **[só na máquina]**.
- Em máquina de 8 GB, um comando pesado por vez (cargo, vitest, Playwright). Worktree novo nasce sem `node_modules` e sem `rust/target` (o do workspace chega a uns 3 GB).
- Verde se declara lendo a saída inteira ou o código do próprio comando: `| tail` devolve o código do `tail`.
- Prova no PR: o que rodou no portão, com o número de testes, o resultado do `checks` e do `build` e, se mexeu em tela, poucas fotos. Sem GIF nem filme.

## CI e publicação

- **Estado em 17/11/2026: `build` e `checks` ligados.** O CI é o único lugar onde a imagem monta, se confere, se publica e se assina. Com o `build` desligado, nenhuma versão sai, nem a semanal. Ligar, desligar ou rodar à mão (`gh workflow run build.yml -R Novasce/novasce-os`) é de quem administra o repositório.
- `checks.yml` (todo PR e push na `main`): `shell` (shellcheck, versão, instalador), `rust` (fmt, clippy `-D warnings`, testes), `web` (tipos e testes), `preparar-tauri`, `m3-tauri` (Começar, Ajuda, Voz e Sede) e `m3-extra` (guia, travessão, geometria).
- `build.yml` em PR: monta as duas imagens e roda as conferências, sem publicar. Em push na `main`, no sábado às 06:00 UTC (pega o Aurora novo da semana) ou à mão: monta, confere, publica `stable-<variante>` e `AAAA.MM.DD.N-<variante>` no GHCR, assina pelo digest e confere a assinatura com o `cosign.pub` da raiz.
- A versão sai de `.github/scripts/proxima-versao.sh`: lê as tags do GHCR e soma 1 ao N do dia (UTC).
- Push que só muda `**/*.md`, `docs/`, `notas/` ou `instalador/` não monta imagem (`paths-ignore`); PR monta sempre.
- `cancel-in-progress`: dois merges seguidos, o build do primeiro é cancelado e sai uma versão só com os dois.
- A publicação leva uns 40 minutos depois do merge. Teste na máquina começa com a versão já publicada e lida no GHCR (roteiro, item 0.4).
- Na máquina, o `uupd` busca a imagem no domingo às 04:00 e a deixa `staged`; nada reinicia sozinho. `sudo bootc rollback` e reiniciar volta uma versão.

### Notas de versão

- `notas/versoes.json` (esquema em `notas/versoes.schema.json`): uma entrada por versão que merece aviso, com `versao` igual à publicada, `tipo` (`importante` ou `opcional`), `linha` e `custo` de 1 a 140 caracteres, sem travessão.
- A barra lê as notas da `main` no GitHub (`novasce-casa/src/config_casa.rs`) e, sem rede, a cópia da imagem. Por isso a nota entra depois que o build publica a versão, num PR só de `notas/`, que não monta imagem. As Novidades da Ajuda leem só a cópia da imagem: lá a nota aparece na montagem seguinte.
- Versão sem nota avisa "Correções e melhorias" como opcional. Uma entrada fora do formato derruba o arquivo inteiro e a barra cai na cópia da imagem.

## Armadilhas

### Imagem e montagem
- `ID=aurora` fica no `os-release`: o `ujust` e as ferramentas da base dependem dele. Só nome, versão e logo mudam (`build_files/10-marca.sh`).
- A política de assinatura se mescla, não se troca: `ghcr.io/ublue-os` continua confiável e o padrão segue `reject` (`50-assinatura.sh`, conferido em `tests/imagem.sh`).
- Sem `cosign.pub` na raiz a montagem para: a imagem leva a chave pública para conferir as próximas.
- Assinatura no formato antigo de propósito (`--new-bundle-format=false --use-signing-config=false`): é o que o rpm-ostree e o containers/image conferem hoje.
- Chave de assinatura com senha precisa do `COSIGN_PASSWORD` no passo Assinar; sem ele, "decryption failed" com a imagem já publicada sem assinatura, que as máquinas recusam.
- Trocar a chave de assinatura tranca as máquinas na versão em uso (elas conferem a imagem nova com a chave da imagem atual): só com um plano de transição.
- `build_files/conferir-aurora.sh` para a montagem quando o Aurora muda um caminho que a casa substitui: o build de sábado pode quebrar sem código novo, e as máquinas seguem na versão anterior.
- No estágio `apps`, `/root` aponta para `/var/roothome`, que não existe na montagem: o `Containerfile` cria a pasta, senão cargo e npm não têm casa.
- `grep -q` num pipe com `pipefail` faz quem escreve antes morrer de SIGPIPE (141): nos scripts de montagem, `grep ... >/dev/null`.
- Esquema de cor copiado do Breeze traz os `Name[xx]` traduzidos: sem apagar, em português ele aparece como um segundo Breeze (`30-kde.sh`).
- O layout do painel e os padrões do look-and-feel só valem para conta nova. Conta que já existia muda por script em `system_files/usr/share/plasma/shells/org.kde.plasma.desktop/contents/updates/`, que o Plasma roda uma vez por conta.
- Os apps Tauri têm `[workspace]` e `Cargo.lock` próprios, e tudo compila com `--locked`: mexeu em dependência de um crate de `rust/crates/` que um app usa, o lock do app muda no mesmo PR (`cargo metadata --manifest-path apps/<app>/src-tauri/Cargo.toml >/dev/null` atualiza sem compilar).
- Clippy limpo no macOS não garante clippy limpo no CI: lá entra o código atrás de `cfg(target_os = "linux")`.

### Máquina e sessão
- Entrada automática sem `Relogin=true`: com ele, o Trocar usuário entrava de novo na Sede em vez de mostrar o login. Quem devolve a Sede é a guarda (`novasce-sede-guarda`).
- A tela de login do Plasma não tem lista de contas escondidas: só some quem tem shell `nologin` ou `false`, e a entrada automática não olha o shell (`novasce-helper/src/acoes.rs`).
- Escolher no Trocar usuário uma sessão aberta pela entrada automática abre uma segunda sessão, que trava: voltar é ativar a que existe (`novasce-voltar-a-sede`, o mesmo `ativar` da tranca).
- Conta nova nasce com a senha que a pessoa digitou: senha vazia vencida (`passwd -d` e `chage -d 0`) não passa pela tela de login do Plasma.
- O Chrome do Flatpak não abre `file:///usr/share`: página local vai para `~/.var/app/<app>/` (`system_files/usr/libexec/novasce/sala`). Em quiosque, `--password-store=basic`, senão a carteira do KDE abre por cima.
- A receita do TPM no Aurora é `ujust toggle-tpm2`; o nome antigo sumiu (`tests/imagem.sh` confere).
- `systemctl reboot` pelo terminal recusa com outra conta logada: `systemctl reboot -i`.
- `sudo bootc rollback` duas vezes sem reiniciar no meio desfaz o pedido ("Reverting queued rollback state"): uma vez, e reiniciar.
- Script servido por `curl | bash` chega pela entrada padrão: comando que lê a entrada dentro dele leva `</dev/null`, senão come o resto do script. Script que pergunta (o `gravar-pendrive.sh`) roda com `bash <(curl ...)`.

### Instalador e testes
- O `anaconda-iso` do bootc-image-builder, sem kickstart próprio, apaga todos os discos (`clearpart --all` e `autopart`, sem cifra). O kickstart da casa vai sempre e não tem linha de disco; `tests/instalador.test.sh` recusa `clearpart`, `autopart`, `part`, `zerombr` e `ignoredisk`.
- O builder faz o `bootc switch` sem conferir assinatura; o `%post` da casa vem depois e troca para `--enforce-container-sigpolicy`.
- Pacote novo no GHCR nasce privado: sem "Public", a máquina não baixa (roteiro, item 0.3).
- Token falso de teste se gera na hora: repetido tem entropia baixa e o gitleaks não dispara; escrito no repositório, a proteção de push do GitHub o toma por real (`tests/imagem-m3.sh`).

## Convenções

- Peça nova na imagem: compila no estágio `apps` (`Containerfile` ou `build_files/apps-m3.sh`), instala em `/out`, liga a unidade em `build_files/32-m3-servicos.sh` ou `60-servicos.sh` e ganha uma linha em `tests/imagem.sh` ou `tests/imagem-m3.sh`.
- Script de montagem novo: `build_files/NN-assunto.sh`, com `set -euo pipefail`, falhando de propósito quando falta algo. O `rodar.sh` só roda `[0-9][0-9]-*.sh`, em ordem.
- A imagem não carrega estado de máquina: papel, registro e contas nascem em `/etc` e `/var` pelo Preparar e pelo `novasce-helper` (fontes em `/usr/share/novasce/sede/`); pasta de estado nasce por `tmpfiles.d`, nunca em `system_files/var/`.
- Config: padrão em `/usr/share/novasce/config.toml` e `apps.toml`; a máquina reforça em `/etc/novasce/` só a chave que muda. O `config.toml` recusa chave desconhecida: chave nova entra no `Config` de `novasce-core/src/config.rs` no mesmo PR.
- Ação de root é subcomando do `novasce-helper`, chamado por `pkexec` (política `com.novasce.helper`: só `wheel`, local e ativo). O serviço root por socket (`novasce-sinal-local`) sabe quem pediu pelo `SO_PEERCRED`, nunca por campo da mensagem.
- Segredo nunca na linha de comando (qualquer conta lê `/proc/<pid>/cmdline`): entrada padrão ou arquivo 0600 (`novasce-casa/src/arquivo_privado.rs`). Tipo que carrega token tem `Debug` escrito à mão, para o token não ir para log.
- Contrato com o Tenancy em camelCase. A máquina pode passar semanas numa versão: campo novo que o OS lê entra com `#[serde(default)]`, e as rotas `/os/*` não mudam de forma.
- Apps Tauri: comando novo em `src-tauri/src/`, registrado no `generate_handler!`; no Começar e na Ajuda, o par em `src/ponte.ts` e `src/ponte-falsa.ts`. `VITE_PONTE=falsa npm exec -w apps/comecar -- vite` abre a tela no navegador, sem Tauri, e `?tela=` escolhe o passo.
- Interface: hover, foco, pressionado e seleção só mudam cor, fundo, contorno e sombra (`propriedadesProibidasEmEstados`, de `@novasce/ui/teste`). Cores e movimento saem de `ui/src/tokens.ts` e `tokens.css`.
- Texto de tela em português e sem travessão. Nada novo nasce com o ponto do meio como separador; a tela que for mexida sai sem ele, e o teste que confere o texto muda junto.
- Logo não se gera nem se redesenha: o cata-vento sai de `marca/catavento.svg` e os ícones dos apps de `marca/apps/compor.py` (README da pasta).
- Nada reinicia sozinho: o `uupd` só prepara e o aviso de atualização pede; reinício só pelo botão da pessoa.

## O que não fazer

- Pôr segredo na imagem ou no repositório. Valor que não pode ser público não entra nem por variável de build; `cosign.key` fica fora (está no `.gitignore`).
- Montar a imagem ou compilar os apps Tauri num Mac: a imagem é x86_64 e sairia em horas, por emulação.
- Mudar o `ID=aurora`, trocar a política de assinatura inteira ou pôr linha de disco no kickstart.
- Mesclar, rodar o `build` à mão, ligar ou desligar workflow ou trocar a chave de assinatura sem o OK de quem administra o repositório.
