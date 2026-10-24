# Voz · medição num notebook sem NVIDIA

Esta medição é feita na sede, depois da imagem do M3 publicada, num notebook sem placa NVIDIA, com o Eduardo presente. Os campos em branco são preenchidos com o que for medido. Nada aqui foi medido ainda.

Data: AAAA-MM-DD · Máquina: <modelo da CPU> · <núcleos> núcleos · <RAM> · Novasce OS <versão>

## Critério (escrito antes de medir)

| Medida | Passa se |
|---|---|
| Fala de ~5 s, em regime (3ª a 5ª rodada), modelo `small-q5_1` | p50 ≤ 2,5 s e p95 ≤ 4,0 s |
| Primeira rodada depois de reiniciar (a frio) | ≤ 8 s |
| Vocabulário: "Novasce", "Tenancy" e "SOPEC" escritos certo | em 2 das 3 frases |
| Shift+Insert cola em Chrome (Tenancy e Gmail), Obsidian, LibreOffice Writer, Konsole, Kate e KRunner | em todos, ou os que falharem entram em "Problemas comuns" |

## Decisão

- Passou com `small`: nada muda.
- Falhou com `small` e passou com `base-q5_1`: `[voz] modelo = "/usr/share/novasce/voz/ggml-base-q5_1.bin"` em `system_files/usr/share/novasce/apps.toml`.
- Falhou com os dois: `[voz] qualquer_campo = false`. O atalho não é registrado e o ditado fica no botão Falar dos apps da casa, como o Pedir ajuda.

Valor padrão até a medição: `qualquer_campo = true` e modelo `small-q5_1`, como já está no `apps.toml`.

## Como medir

Num terminal do notebook (é a única vez que alguém usa terminal nesta tarefa, e é quem mede).

1. Gravar três frases de uns 5 segundos cada:

```bash
for n in 1 2 3; do echo "Frase $n: fale por uns 5 segundos e espere"; parecord --file-format=wav --rate=16000 --channels=1 --format=s16le "/tmp/frase$n.wav" & p=$!; sleep 6; kill $p; done
```

Frases: (1) "Abri o Tenancy e conferi o SOPEC da Gotec hoje de manhã." (2) "A Novasce manda a planilha nova até sexta." (3) "Mandei a nota no brain e avisei no GitHub."

2. Medir a frio (logo depois de reiniciar, sem abrir nada) e em regime:

```bash
sudo systemctl reboot
novasce-voz-medir --arquivo /tmp/frase1.wav --vezes 5
novasce-voz-medir --arquivo /tmp/frase2.wav --vezes 5
novasce-voz-medir --arquivo /tmp/frase3.wav --vezes 5
novasce-voz-medir --arquivo /tmp/frase1.wav --vezes 5 --modelo /usr/share/novasce/voz/ggml-base-q5_1.bin
grep -m1 "model name" /proc/cpuinfo; nproc; free -h | head -2
```

Cada rodada imprime `rodada N: X ms · Ok("…")` e a última linha traz p50 e p95. A "a frio" é a rodada 1 logo depois de reiniciar.

3. Conferir a colagem: com o cursor em cada app da tabela, segurar Super + Espaço, falar "teste de colagem" e soltar. Anotar "colou" ou "não colou".

## Resultados

| Modelo | Frase | A frio (ms) | p50 (ms) | p95 (ms) | Texto |
|---|---|---|---|---|---|
| small-q5_1 | 1 | | | | |
| small-q5_1 | 2 | | | | |
| small-q5_1 | 3 | | | | |
| base-q5_1 | 1 | | | | |

| App | Shift+Insert |
|---|---|
| Chrome (Tenancy) | |
| Chrome (Gmail) | |
| Obsidian | |
| LibreOffice Writer | |
| Konsole | |
| Kate | |
| KRunner | |

Decisão: <small fica | troca para base | qualquer_campo = false>, pelo critério acima.

Se algum app não colou, acrescentar em `problemas-comuns` (`ajuda/conteudo/guias.ts`) a linha "O ditado não escreveu no campo do <app>." com a resposta "O texto ficou copiado. Cole com Ctrl+V.".
