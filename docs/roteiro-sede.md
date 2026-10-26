# Roteiro do teste na sede · Novasce OS M1

## TLDR

- O que se prova: a sede instala pelo pendrive, troca para a imagem assinada da Novasce, mostra a marca no boot e no login, passa pelo Preparar esta máquina, entra sozinha na conta Sede com a Sala e atualiza e volta de versão.
- Quem faz: o Eduardo, sentado na máquina da sede (RTX 2060).
- Quanto tempo: umas 2 horas, com a instalação.
- Marcado com **[só na máquina]**: o que o CI não consegue provar e só se confirma olhando a tela.
- Sem a imagem publicada: os itens 0.5 e 0.6, o passo 0 (se a máquina já tem sistema), os passos 1 a 3 e o 3b, que confere a Sala no navegador.

## Antes de sair de casa

| # | Conferir | Como | Resultado esperado |
|---|---|---|---|
| 0.1 | Repositório público e chave criada (M0, itens 1 e 2) | `gh repo view Novasce/novasce-os --json visibility` | `PUBLIC`, e `cosign.pub` no `main` |
| 0.2 | As duas imagens publicadas e assinadas | `cosign verify --key cosign.pub ghcr.io/novasce/novasce-os:stable-nvidia` | `Verification for ghcr.io/novasce/novasce-os:stable-nvidia` sem erro |
| 0.3 | O pacote do GHCR está público | github.com/orgs/Novasce/packages/container/novasce-os/settings, "Change visibility" | `Public`. Sem isso, a máquina não baixa a imagem |
| 0.4 | A versão publicada | `skopeo inspect docker://ghcr.io/novasce/novasce-os:stable-nvidia \| jq -r '.Labels["org.opencontainers.image.version"]'` | `AAAA.MM.DD.N` |
| 0.5 | Pendrive de **16 GB ou mais** com o ISO `aurora-nvidia-open-stable-x86_64.iso` (M0, item 6). O ISO tem 7,9 GB e não cabe num de 8 GB | getaurora.dev: Aurora, placa NVIDIA, sem DX. É o mesmo `aurora-nvidia-open` de que a imagem da casa parte. Conferir com `shasum -a 256` contra o arquivo `-CHECKSUM` ao lado do ISO e gravar no Fedora Media Writer (ou `dd` no `/dev/rdiskN` do pendrive, no Mac) | o checksum bate e o pendrive aparece como "Aurora" |
| 0.6 | Senha forte para o disco e senha de administrador do BIOS (passo 1) | no gerenciador de senhas, nunca no brain | as duas anotadas |

## 0. Antes de apagar o sistema que já está lá

Só se a máquina já tem um sistema. Em 10/2026 a sede ainda tem o Fedora antigo, e o passo 2 apaga o disco inteiro.

1. Entrar na máquina e abrir um terminal.
2. `ls /home` e `du -sh /home/* 2>/dev/null`: o que importar sai antes, para o Drive ou para o Mac (`scp`). O pendrive não serve: ele é o instalador.
3. `ls ~/.ssh`: se a chave desta máquina foi cadastrada no GitHub (Settings, SSH keys), apagar de lá depois da instalação. A chave morre com o disco.
4. A sede não tem cabo de rede: a internet vem de um adaptador Wi-Fi USB. Anotar o modelo (`lsusb`) e o driver: `nmcli device` dá o nome do Wi-Fi e `modinfo -F filename $(ethtool -i <nome> | awk '/^driver/{print $2}')` mostra de onde ele vem.

**Esperado:** nada a salvar, ou tudo copiado para fora. O driver do Wi-Fi mora em `.../kernel/drivers/...`, que vem no Aurora. Se morar em `extra` ou `updates`, foi instalado à parte e o Aurora não terá: levar o cabo USB do celular e usar a internet dele pelo USB (ancoragem) no 3b.

## 1. BIOS

1. Ligar e entrar no setup do BIOS (Del ou F2).
2. Ligar o TPM (Intel PTT ou AMD fTPM).
3. Ligar o Secure Boot no modo padrão.
4. Pôr senha de administrador no BIOS.
5. Ordem de boot: o pendrive primeiro.

**Esperado:** a tela de segurança mostra TPM 2.0 ligado e Secure Boot "Enabled".

## 2. Instalar o Aurora NVIDIA

1. Dar boot pelo pendrive e escolher instalar.
2. Usar o disco inteiro e apagar tudo. Se a máquina tiver um segundo disco (o do Windows, em 10/2026 um SSD à parte para jogos), escolher só o principal e deixar o outro desmarcado.
3. Marcar **Criptografar meus dados** e usar a senha do item 0.6.
4. Criar o usuário `eduardo` como **administrador**.

**Esperado:** a instalação termina e pede para reiniciar.

## 3. Primeiro boot e a chave da Universal Blue (MOK)

1. Tirar o pendrive e reiniciar.
2. Na tela azul do MOK: **Enroll MOK**, **Continue**, senha `universalblue`, **Reboot**.
3. Digitar a senha do disco e entrar como `eduardo`.
4. Num terminal: `mokutil --sb-state` e `nvidia-smi`.

**Esperado:** boot com a marca do Aurora; `SecureBoot enabled`; `nvidia-smi` mostra a GeForce RTX 2060.

## 3b. Sem a imagem: a Sala no navegador

Só enquanto a imagem da Novasce não está publicada. Com a imagem, pular para o passo 4: a Sala abre sozinha no passo 8. Precisa do M2 no ar (desde 25/10) e do celular com o Tenancy.

Antes, ligar o Wi-Fi pelo ícone de rede da bandeja. Se o adaptador não aparecer, usar a internet do celular pelo cabo USB (ancoragem).

1. Na conta `eduardo`, abrir o Firefox, que vem no Aurora (se não houver, instalar pela Discover), em `tenancy.novasce.com/sala`. F11 deixa em tela cheia.
2. No celular, abrir **Máquinas e telas** no Tenancy. Em **Pedidos novos**, conferir que o código é o da TV, clicar **Aprovar** e manter o nome "Tela da sede".
3. Esperar a TV trocar sozinha. Ela pergunta a cada 3 segundos.
4. Na TV, clicar **Ver tudo**, ler o QR com o celular, entrar e tocar em **Mostrar na TV**.
5. Fechar o Firefox e abrir de novo em `tenancy.novasce.com/sala`.
6. Em **Máquinas e telas**, seção **Telas**, clicar **Revogar** na "Tela da sede" e confirmar.

**Esperado:**

- Passo 1: "Conectar esta tela à Sala", um código de 6 letras e "Esperando aprovação · o código vale 10 minutos". Código vencido troca sozinho por um novo.
- Passo 2: o aviso "Tela aprovada. A Sala aparece nela em instantes."
- Passo 3: a Sala no modo discreto, sem dinheiro nem cliente, com os blocos Agentes, Quadro de hoje, Saúde dos sistemas, Números e Mural. Agentes vazio é o esperado: o executor só sobe na sede com a imagem (passo 11). Na Saúde, SOPEC ou Tenancy sem alvo quer dizer que o Endereço de produção do produto está vazio (Produtos, aba Técnico).
- Passo 4: "Ver tudo · esperando o login no celular" e, depois do toque, "Ver tudo · volta ao discreto em" com a contagem dos 10 minutos e o botão **Voltar ao discreto**.
- Passo 5: a Sala abre já pareada, sem pedir código. O token fica no navegador.
- Passo 6: "Tela revogada. Ela volta a mostrar o código para conectar." Na leitura seguinte, a TV mostra "O acesso desta tela foi revogado. Peça a aprovação de novo com o código abaixo."

Se a TV ficar na Sala até a imagem chegar, pular o passo 6 e, nas Configurações do Sistema da conta `eduardo`, desligar o apagar da tela, a suspensão e o bloqueio automático. Quando a imagem chegar, revogar esta tela: a conta Sede pareia a dela no passo 8.

## 4. Trocar para a imagem da Novasce

```bash
sudo bootc switch --enforce-container-sigpolicy ghcr.io/novasce/novasce-os:stable-nvidia
sudo bootc status
```

**Esperado:** o download termina com `Queued for next boot`; `bootc status` mostra `ghcr.io/novasce/novasce-os:stable-nvidia` em **staged**.

Por que passa já na primeira vez: o `--enforce-container-sigpolicy` exige que a política da máquina recuse por padrão o que não tem regra, e a do Aurora recusa. Esta primeira troca confia no TLS do GHCR, porque o Aurora ainda não conhece a chave da Novasce. A partir do próximo boot, a política da própria imagem confere a chave da casa em toda atualização (passo 6).

## 5. Reiniciar e ver a marca

```bash
systemctl reboot
```

**Esperado [só na máquina]:**

- Boot: fundo teal, cata-vento branco **parado** no centro e uma linha fina que enche. Nenhum ícone girando, nenhum pontinho.
- Pedido da senha do disco (o TPM ainda não está ligado): o mesmo fundo teal, o texto "Digite a senha do disco" e um ponto por letra.
- Tela de login: o papel de parede teal com o cata-vento gigante tom sobre tom.
- Depois do login, a abertura: teal, cata-vento parado e a linha que anda até o fim.
- Área de trabalho B: barra fina no alto (cata-vento no menu, menu do app, bandeja e relógio) e dock flutuante embaixo com os apps.

## 6. Conferir a assinatura

```bash
rpm-ostree status | head -n 5
jq '.transports.docker["ghcr.io/novasce/novasce-os"]' /etc/containers/policy.json
sudo bootc upgrade --check
grep IMAGE_VERSION /usr/lib/os-release
```

**Esperado:** `ostree-image-signed:docker://ghcr.io/novasce/novasce-os:stable-nvidia`; a regra `sigstoreSigned` com `/usr/lib/pki/containers/novasce-os.pub`; o `upgrade --check` sem erro de assinatura; `IMAGE_VERSION` igual ao item 0.4.

## 7. Preparar esta máquina

O Preparar abre sozinho no login do `eduardo` (administrador). Se não abrir, procurar "Preparar esta máquina" no menu.

| Passo | O que fazer | Esperado |
|---|---|---|
| 7.1 Papel | Escolher **Sede** e Continuar | A linha diz "Placa de vídeo encontrada: NVIDIA GeForce RTX 2060. Esta máquina usa a versão com NVIDIA." O KDE pede a senha de administrador uma vez |
| 7.2 Rede | Com cabo, Continuar | "Esta máquina já está na internet, pelo cabo." |
| 7.2b Rede por Wi-Fi (sem cabo; na sede é o caminho, porque lá não há cabo) | Tirar o cabo, escolher a rede, digitar a senha e clicar **Conectar** | "Esta máquina já está na internet, pela rede" e o nome dela. Com a senha errada: "Não conectou na rede" e o nome, e a rede não fica em `nmcli connection show` |
| 7.3 Registro, sem o M2 no ar | Esperar a resposta | "O Tenancy ainda não recebe máquinas." Clicar **Registrar depois** |
| 7.3 Registro, com o M2 no ar | Aprovar o código de 6 letras em **Máquinas e telas** no Tenancy | A tela avança sozinha para o passo 4. `sudo ls -l /var/lib/novasce/maquina.json` mostra `-rw-------` e dono `root` |
| 7.3b Código recusado (teste opcional) | Gerar um código e recusar no Tenancy | "O Tenancy recusou este código. Gere outro e peça a aprovação de novo." e o botão **Gerar outro código** (a API respondeu `403`) |
| 7.3c Código vencido (teste opcional) | Gerar um código e esperar 10 minutos sem aprovar | "O código venceu ou já foi usado. Gere outro código." (a API respondeu `410`) |
| 7.4 De quem é | Criar uma conta de teste ("Teste Sede", `teste@novasce.com`) e clicar **Concluir a sede** | "Contas criadas: teste." e a tela "Máquina pronta" |

Conferência do Wi-Fi (só se o 7.2b foi feito). A senha não pode passar pela linha de comando, onde qualquer conta da máquina a leria. Em outro terminal, durante o clique em **Conectar**:

```bash
watch -n 0.2 'pgrep -af nmcli'
```

**Esperado:** nenhuma linha traz a senha nem a palavra `password`; a ativação aparece com `passwd-file /run/user/<uid>/novasce-wifi-.../senha`. Depois de conectar:

```bash
nmcli -s connection show "<nome da rede>" | grep psk
ls /run/user/$(id -u) | grep novasce-wifi || echo "nenhum arquivo de senha sobrou"
```

**Esperado:** `802-11-wireless-security.psk` com a senha digitada (o `-s` mostra o segredo de propósito: não copiar essa saída para lugar nenhum), `psk-flags: 0`, e "nenhum arquivo de senha sobrou". A senha salva é o que reconecta a máquina: no passo 8, depois de **Reiniciar agora**, sem cabo, a máquina volta sozinha ao Wi-Fi, sem pedir a senha (`nmcli -t -f ACTIVE,SSID device wifi | grep '^yes'` mostra a rede) **[só na máquina]**.

Conferir no terminal:

```bash
cat /etc/novasce/papel
getent passwd sede teste
id teste
cat /etc/plasmalogin.conf.d/50-novasce-sede.conf
systemctl status novasce-sinal.timer --no-pager
```

**Esperado:** `sede`; as duas contas; `teste` **sem** `wheel`; `[Autologin]` com `User=sede`; o timer do sinal ativo.

Quantas vezes a senha de administrador foi pedida no Preparar **[só na máquina]**: o esperado é uma vez a cada 5 minutos. Anotar se pediu mais.

## 8. A sede entra sozinha na Sala

Clicar **Reiniciar agora**.

**Esperado [só na máquina]:**

- Depois do boot, nenhuma tela de login: a conta Sede entra sozinha.
- O Chrome abre em tela cheia. Sem o M2: a página "A Sala chega com o Tenancy". Com o M2: a Sala, que pede o pareamento da tela na própria página.
- Com o M2: parear a TV na página da Sala e reiniciar. Depois do boot, a Sala abre já pareada, sem pedir código de novo (o token fica no perfil do Chrome da Sede; o sistema não guarda token de tela).
- No primeiro boot, o Chrome pode levar alguns minutos para aparecer (está sendo instalado). Enquanto isso, fica o papel de parede teal.
- A tela não apaga sozinha e a máquina não suspende.

## 9. Conta pessoal sem sudo

1. No menu da Sede, **Trocar usuário** e entrar como `teste`.
2. Com a senha vazia, o login pede para criar a senha nova.
3. Entrar, abrir o Konsole e rodar `sudo -v`.
4. Ainda no Konsole: `xdg-settings get default-web-browser` e `xdg-mime query default x-scheme-handler/https`.
5. Clicar num link qualquer (por exemplo, no "Sobre este sistema").

**Esperado [só na máquina]:** o login pede a senha nova (a conta nasce com a senha vencida); o `sudo -v` responde que `teste` não está no sudoers; o avatar do login mostra a inicial "T" no círculo teal claro; os dois comandos respondem `com.google.Chrome.desktop` e o link abre no Chrome, não no Firefox.

Se o login não pedir a senha nova (limite do plasmalogin), o Eduardo define com a pessoa ao lado: `sudo passwd teste`. Anotar para corrigir no M3.

## 10. Voltar à Sede

Trocar usuário de volta para **Sede**.

**Esperado [só na máquina]:** volta com Enter, sem senha, e a Sala continua aberta.

## 11. Agentes

Na conta `eduardo`:

```bash
sudo novasce-primeiro-uso
```

Seguir os passos impressos (chaves de deploy, login do Claude, setup, doctor). Só com o doctor verde:

```bash
sudo systemctl enable --now novasce-assistente.timer
systemctl status novasce-assistente.timer --no-pager
sudo flock -n /run/novasce-agentes/rodada.lock true && echo "fora de uma rodada"
```

**Esperado:** timer ativo; `runner.log` crescendo em `/var/lib/novasce-agentes/assistentes/state/`; durante uma rodada o `flock` não consegue a trava, e entre rodadas imprime "fora de uma rodada".

## 12. Disco pelo TPM

```bash
ujust setup-luks-tpm-unlock
systemctl reboot
```

**Esperado:** o boot não pede mais a senha do disco e chega sozinho à Sala.

## 13. Sinal

```bash
sudo systemctl start novasce-sinal.service
journalctl -u novasce-sinal -n 5 --no-pager
```

**Esperado:** com o M2 no ar e a máquina registrada, `Sinal enviado: versão AAAA.MM.DD.N, ... disco cifrado true, Secure Boot true`. Sem o M2, `O Tenancy ainda não recebe o sinal`. Sem registro, o serviço nem roda (falta o `maquina.json`).

Para registrar depois que o M2 subir: `sudo novasce-sinal pair` e aprovar o código no Tenancy.

## 14. Atualização

Quando o CI publicar uma versão nova (ou forçando a busca):

```bash
sudo systemctl start uupd.service
rpm-ostree status
sudo novasce-sinal heartbeat
```

**Esperado:** `rpm-ostree status` mostra a versão nova como **staged**; nada reinicia sozinho, nem na sede; o sinal leva a versão preparada. Reiniciar e conferir `grep IMAGE_VERSION /usr/lib/os-release`.

**[só na máquina]:** conferir com `rpm-ostree status --json | jq '.deployments[].version'` que a versão aparece no formato `AAAA.MM.DD.N`. Se vier vazia, anotar: o Sinal usa o começo do checksum no lugar.

## 15. Voltar de versão

```bash
sudo bootc rollback
systemctl reboot
grep IMAGE_VERSION /usr/lib/os-release
```

**Esperado:** a máquina volta para a versão anterior. Para ir de novo para a mais nova: `sudo bootc rollback` e reiniciar.

## 16. Tema escuro e menos movimento

1. Configurações do Sistema, Cores, **Novasce Grafite**.
2. Configurações do Sistema, Animações, velocidade **Instantâneo**, e reiniciar a sessão.

**Esperado [só na máquina]:** fundo grafite `#1E1E20` com destaque ciano; na abertura, a linha aparece sem animar.

## O que só se confirma na máquina

| Item | Por que o CI não prova |
|---|---|
| Entrada automática da Sede (`[Autologin]`, `Session=plasma.desktop`, `Relogin=true`) | o plasmalogin só roda com tela |
| Layout B: alturas de 30 e 62 px, dock que se ajusta aos apps (`lengthMode = "fit"`) e raio de 8 px | o script de layout só roda no Plasma; o raio vem do tema Breeze |
| A linha da abertura chega ao fim (estágios 1 a 6 do KSplash) | precisa do KSplash de verdade |
| O pedido da senha do disco no Plymouth | precisa do boot real |
| Ícones do dock agrupando as janelas do Chrome (`StartupWMClass`) | depende do Chrome em execução |
| A janela do Preparar abre com a NVIDIA (`WEBKIT_DISABLE_DMABUF_RENDERER=1`) | precisa da placa |
| Quantas vezes a senha de administrador é pedida no Preparar | depende do agente do polkit no KDE |
| A conta nova pede a senha no primeiro login | depende do plasmalogin |
| O avatar com a inicial aparece no login | depende do accounts-daemon |
| Uma conta nova nasce clara com destaque teal | depende do Plasma aplicando o perfil |
| O Chrome é o navegador padrão de uma conta nova (`BrowserApplication` e `mimeapps.list`) | depende da ordem do `XDG_CONFIG_DIRS` na sessão |
| A Sala continua pareada depois de reiniciar (perfil persistente do Chrome) | depende da página `/sala` do M2 |
| A tela da Sede não apaga (`powerdevilrc`) | precisa da sessão |
| A versão no `rpm-ostree status --json` | só existe numa máquina com deployments |

## Depois do M3 publicado

Piloto: o Eduardo, no computador da empresa. Cada passo diz o que fazer e o que deve aparecer.

| # | O que fazer | O que deve aparecer |
|---|---|---|
| 1 | Entrar na conta pessoal pela primeira vez | O Começar abre sozinho no passo 1, com um código |
| 2 | Entrar no Tenancy pelo botão e aprovar o código | O seu nome no alto e o primeiro terço do contorno fechado |
| 3 | Marcar Claude e clicar em Instalar e entrar | "Instalando o Claude Code", depois o login do Claude no navegador |
| 4 | Aprovar o código no GitHub e clicar em Criar meu brain | `<login>-brain` privado no GitHub e a tela dos espaços |
| 5 | Trazer o Studio | Pasta `~/Espaços/Studio` e "Pronto. Os três passos estão feitos." |
| 6 | Conectar o Google no fim | Pasta "Novasce OS/Documentos" no Drive com o que estava em Documentos |
| 7 | Fechar o Começar e entrar de novo | O Começar não abre; ele saiu do dock; volta pela Ajuda |
| 8 | Sair e entrar na manhã seguinte | O Meu dia do Tenancy abre uma vez |
| 9 | Clicar no "?" com o Dolphin na frente | A Central com "Nesta tela · Arquivos" |
| 10 | Pedir ajuda com print | "Chamado aberto" e o chamado no Tenancy com print INTERNAL e dados da máquina |
| 11 | Responder o chamado no Tenancy com tipo Solução | Aviso "A Novasce respondeu" com Resolveu e Ainda não |
| 12 | Desligar o Wi-Fi e pedir ajuda | "O chamado não foi enviado", e ele sai sozinho ao religar |
| 13 | Escrever uma nota no Obsidian | Em até 3 min, a nuvem verde com "salvos há N min" |
| 14 | Pôr um token falso numa nota | Nuvem laranja dizendo que a nota ficou só neste computador |
| 15 | Voz: Tarefa 35 inteira | Números em `docs/medicoes/voz.md` e decisão aplicada |
| 16 | Conta Sede: abrir o Navegador da Sede, logar num site e não mexer 10 min | Aviso com 30 s, Sala em tela cheia; o navegador abre de novo sem o login |
| 17 | Conta pessoal na sede: não mexer 5 min | A conta tranca e a TV volta à Sala; ao entrar de novo, os apps continuam abertos |
| 18 | Montar uma versão de teste e deixar pronta | Aviso uma vez; "Depois" deixa o sinal "x pronta"; na Sede, o canto com Reiniciar só fora de rodada |
| 19 | Tela de login | Cartão com o QR no canto; o celular abre `/os/esqueci?m=…` |
| 20 | KDE: velocidade de animação em "Instantânea" e abrir o Começar | O terço aparece pintado, sem movimento |

Premissas para conferir aqui (do plano do M3): fundo por máquina em `/etc/plasmalogin.conf.d/`, KWallet abrindo sozinho, painel da voz como camada sem roubar foco, Chrome Flatpak e título "Sala da Novasce", acesso do OAuth App aos repositórios dos espaços, cliente do Google fora do modo de teste, `Meta+Espaço` livre.

Conferências que a execução do M3 levantou (não dava para provar no Mac):

- QR do login: o `rsvg-convert` carrega o papel de parede de `/usr/share/wallpapers/...` dentro do SVG gravado em `/var/lib/novasce-login/`? Se o fundo sair sem o papel, copiar o papel para a mesma pasta do SVG.
- Sinal de senhas: o clique esquerdo abre o Bitwarden e o menu com o Cofre do time fica no clique direito (o `ksni` 0.2 não abre menu no clique esquerdo).
- Canto "atualização pronta" na Sede: 72 px de altura; com o motivo do botão em duas linhas o texto encosta na borda. Se incomodar na TV, subir para 88 px.
- Voz: preencher `docs/medicoes/voz.md` (Tarefa 35) antes de prometer "qualquer campo".
