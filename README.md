# Novasce OS

## TLDR

- Sistema da máquina da sede: servidor dos agentes e computador do time, no mesmo disco.
- É o Aurora (KDE, Universal Blue, Fedora) com driver NVIDIA aberto, mais a marca e os serviços da Novasce. A receita inteira está neste repositório.
- Atualização semanal, montada na própria máquina. Fica pronta e só entra quando alguém reinicia. Se der errado, `sudo bootc rollback` volta para a versão anterior.
- Nenhum segredo entra na imagem. O cofre dos agentes é criado na máquina.

## O que tem aqui

| Arquivo | Para quê |
|---|---|
| `Containerfile` | A receita: base Aurora + `system_files/` + `build.sh` |
| `build.sh` | Pacotes, marca (papel de parede, boot, abertura, tema, nome) e initramfs |
| `marca/` | Cata-vento oficial (cópia de `design/novasce/marca-vetor`) e o papel de parede |
| `system_files/` | Arquivos copiados para a raiz: usuário `agentes`, serviços, scripts, tela de login |

## Instalar (uma vez, sentado na máquina)

**Antes:** pendrive de 8 GB ou mais. A senha do disco fica no gerenciador de senhas, nunca no brain.

1. **BIOS:** TPM ligado (Intel PTT ou AMD fTPM), Secure Boot ligado, senha de administrador no BIOS.
2. **Pendrive:** baixar o Aurora em getaurora.dev, na versão **NVIDIA** (placas GTX 16xx, RTX 20xx ou mais novas), e gravar com o Fedora Media Writer.
3. **Instalação:** usar o disco inteiro e apagar tudo. Marcar **Criptografar meus dados**. Criar o seu usuário como administrador.
4. **Primeiro boot:** se aparecer a tela azul do MOK, escolher *Enroll MOK* → *Continue* → senha `universalblue` → reiniciar. Conferir com `nvidia-smi`.
5. **Trazer a receita** (num terminal da máquina):

   ```sh
   sudo mkdir -m 0700 -p /var/lib/novasce-os
   sudo ssh-keygen -q -t ed25519 -N "" -C "sede: novasce-os" -f /var/lib/novasce-os/chave-github
   sudo cat /var/lib/novasce-os/chave-github.pub
   ```

   Adicionar essa chave em github.com/Novasce/novasce-os → Settings → Deploy keys, **sem** "Allow write access". Depois:

   ```sh
   SSH="ssh -i /var/lib/novasce-os/chave-github -o IdentitiesOnly=yes -o StrictHostKeyChecking=accept-new"
   sudo git -c core.sshCommand="$SSH" clone git@github.com:Novasce/novasce-os.git /var/lib/novasce-os/repo
   sudo git -C /var/lib/novasce-os/repo config core.sshCommand "$SSH"
   sudo bash /var/lib/novasce-os/repo/system_files/usr/libexec/novasce/atualizar
   systemctl reboot
   ```

6. **Agentes:** `sudo novasce-primeiro-uso`. Ele confere a máquina, clona brain e executor, instala o Claude, roda os testes e imprime os passos interativos que faltam.
7. **Disco pelo TPM:** `ujust setup-luks-tpm-unlock`, para a máquina ligar sem pedir a senha do disco.
8. **Contas do time:** Configurações do Sistema → Usuários → Adicionar, tipo **Padrão** (nunca administrador).

## Conferir depois de instalar

- `bootc status` mostra `localhost/novasce-os:latest` como booted.
- `nvidia-smi` mostra a RTX 2060. `mokutil --sb-state` diz *SecureBoot enabled*.
- Boot, abertura, tela de login e área de trabalho com a marca. "Sobre este sistema" diz Novasce OS.
- Reiniciar não pede a senha do disco.
- `systemctl status novasce-assistente.timer` ativo, e `state/runner.log` do executor crescendo.
- Uma conta do time não consegue `sudo`.

## Dia a dia

| Quero | Comando |
|---|---|
| Atualizar agora | `sudo systemctl start novasce-atualizar` e reiniciar quando ninguém estiver usando |
| Ver quem está logado antes de reiniciar | `loginctl list-sessions` |
| Ver o que está rodando e o que entra no próximo boot | `bootc status` |
| Voltar para a versão anterior | `sudo bootc rollback` e reiniciar |
| Log da atualização | `journalctl -u novasce-atualizar` |
| Log dos agentes | `journalctl -u novasce-assistente` e `state/` do executor |

## Fora da v0

- Claude e Codex para as contas do time.
- Apps Mac da casa (Tenancy macOS, Sessões, Assuntos): não rodam em Linux. O Tenancy entra pelo navegador.
- Acesso de fora da rede da sede.
