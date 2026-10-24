# M3 · o que o M1 entregou (conferido em 23/10/2026 na branch `m1-base`)

O M1 ainda não foi mesclado: o M3 parte da branch `m1-base` (PR novasce-os#1). Tudo o que o M3 importa do `novasce-core` passa por `rust/crates/novasce-casa/src/nucleo.rs`.

| Peça | Situação |
|---|---|
| Base e erros (contrato v2) | igual ao plano do M1: `tenancy_api` termina em `/api/v1`, Bearer, `403`, `410`, `428` |
| Config | igual: `Config { tenancy_api, tenancy_web, sala_url }`, `carregar()`, `de_texto(imagem, maquina)`, `deny_unknown_fields` |
| Contrato | igual nos nomes. Diferente: `Pessoa.is_new` e `Comecar.returning` são obrigatórios no M1 (sem `#[serde(default)]`). Um Tenancy antigo sem esses campos falha no parse; o teste `interface_m1` manda os dois. Sem ajuste em `nucleo.rs` |
| Token da máquina | igual: `GuardaDoToken::em(..).ler() -> Result<Option<MaquinaSalva>, ErroToken>`, `CAMINHO_DA_MAQUINA` |
| Cliente HTTP | igual: `ClienteTenancy` só com as rotas da máquina; o M3 usa `novasce_casa::cliente::Tenancy` |
| Notas de versão | igual: `novasce_core::notas::{ler_notas, NotaDeVersao, TipoDeNota}`, lista `AAAA.MM.DD.N` |
| Papel | igual: `/etc/novasce/papel` (`novasce_core::sistema::CAMINHO_PAPEL`, `ler_papel`) |
| Conta Sede | igual: `/usr/libexec/novasce/sala`, Chrome Flatpak em quiosque |
| Abrir web | igual: `/usr/libexec/novasce/abrir-web <nome>` e os `.desktop` da casa |
| UI | igual: `Botao`, `Campo`, `Catavento`, `AnelProgresso`, tokens `--nv-*` |
| Workspaces | igual: `["ui", "apps/*"]`; o M3 acrescenta `ajuda` |
| Apps Tauri | igual: molde do Preparar (`apps/preparar/src-tauri`, `[workspace]` próprio, `Cargo.lock` próprio) |
| Painel | igual: `org.kde.plasma.desktop-layout.js` com bandeja e dock `icontasks`; o M3 pôs os sinais em `shownItems` e o Começar no dock |
| Containerfile | igual: estágio `apps` em `/src` gravando em `/out`; `rodar.sh` roda `build_files/[0-9][0-9]-*.sh` |
| CI | igual: `checks.yml` com `shell`, `rust`, `web` e `preparar-tauri` |
| Agentes | igual: `novasce-assistente.service`; a trava da rodada é `/run/novasce-agentes/rodada.lock` (`sistema::agentes_ociosos`) |
| Workspace Cargo | diferente: `rust/Cargo.toml` lista os membros um a um e usa `[workspace.dependencies]`; cada crate do M3 entra em `members` e herda as versões |
