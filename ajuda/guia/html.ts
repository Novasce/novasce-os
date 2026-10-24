import { GUIAS, PALAVRAS, SECOES_DO_GUIA, guiaPorId } from "../index";
import type { Guia } from "../tipos";

export type OpcoesGuia = { versao: string; data: string; catavento: string };

const esc = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

export function recusarTravessao(texto: string): string {
  if (texto.includes("\u2014")) throw new Error("O Guia tem travessão. Troque por ponto, vírgula ou dois-pontos.");
  return texto;
}

function rotulo(g: Guia): string {
  return g.rotuloNoGuia ?? g.titulo;
}

function secao(g: Guia): string {
  const passos = g.passos
    .map(
      (p) => `<div class="passo"><b>${esc(p.titulo)}</b>
<div class="regua"><div><span class="etq">O que fazer</span><p>${esc(p.oQueFazer)}</p></div><div><span class="etq">Onde clicar</span><p>${esc(p.ondeClicar)}</p></div><div><span class="etq">O que deve aparecer</span><p>${esc(p.oQueDeveAparecer)}</p></div></div>
${p.exemplo ? `<div class="exemplo">${esc(p.exemplo)} · Exemplo · dados fictícios</div>` : ""}</div>`,
    )
    .join("\n");
  const problemas = g.problemas?.length
    ? `<table><thead><tr><th>O que aparece</th><th>O que fazer</th></tr></thead><tbody>${g.problemas.map((p) => `<tr><td>${esc(p.oQueAparece)}</td><td>${esc(p.oQueFazer)}</td></tr>`).join("")}</tbody></table>`
    : "";
  return `<section><h2 id="${g.id}">${esc(rotulo(g))}</h2><p class="resumo">${esc(g.resumo)}</p>${passos}${problemas}</section>`;
}

export function gerarHtml(o: OpcoesGuia): string {
  const guias = SECOES_DO_GUIA.map((id) => guiaPorId(id)!).filter(Boolean);
  const nav = guias.map((g) => `<a href="#${g.id}">${esc(rotulo(g))}</a>`).join("");
  const palavras = PALAVRAS.map((p) => `<span><b>${esc(p.termo)}</b> · ${esc(p.definicao)}</span>`).join("");
  const html = `<!doctype html>
<html lang="pt-BR"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>Guia do Novasce OS ${esc(o.versao)}</title>
<style>
:root{color-scheme:light;--ink:#1D1D1F;--ink2:#3B4446;--muted:#647073;--line:#E5E9E8;--line2:#CBD3D1;--teal:#00666C;--soft:#F7F8F7}
*{box-sizing:border-box}body{margin:0;background:#fff;color:var(--ink);font:400 15px/1.55 Inter,-apple-system,Arial,sans-serif}
header{height:64px;border-bottom:1px solid var(--line);display:flex;align-items:center;gap:12px;padding:0 36px}
header svg{width:24px;height:24px;color:var(--teal)}header b{font-size:17px}
.etq{font-family:"JetBrains Mono",ui-monospace,monospace;font-size:11px;letter-spacing:.08em;text-transform:uppercase;color:var(--muted)}
header .etq{margin-left:auto}
.corpo{display:grid;grid-template-columns:250px 1fr;min-height:calc(100vh - 64px)}
nav{border-right:1px solid var(--line);background:var(--soft);padding:20px 22px;display:flex;flex-direction:column;gap:8px;font-size:13px;color:var(--ink2)}
nav a{color:var(--ink2);text-decoration:none}nav a:focus-visible{outline:2px solid var(--teal);outline-offset:3px}
main{padding:26px 40px;max-width:980px}h2{font-size:26px;margin:28px 0 4px}.resumo{color:var(--ink2);font-size:16px;margin:0 0 14px}
.passo{border:1px solid var(--line);border-radius:8px;padding:18px;margin:12px 0;display:flex;flex-direction:column;gap:12px}
.regua{display:grid;grid-template-columns:repeat(3,1fr);gap:14px;font-size:14px;color:var(--ink2)}.regua p{margin:4px 0 0}
.exemplo{height:120px;border:1px dashed var(--line2);border-radius:6px;display:grid;place-items:center;font-size:13px;color:var(--muted);background:#FBFCFC}
table{border-collapse:collapse;width:100%;font-size:14px}th,td{border:1px solid var(--line);padding:10px;text-align:left;vertical-align:top}th{background:var(--soft)}
footer{margin-top:32px;font-size:13px;color:var(--muted);border-top:1px solid var(--line);padding-top:10px}
@media print{@page{size:A4;margin:14mm}.corpo{display:block}nav{display:none}.passo{break-inside:avoid}}
@media (max-width:720px){.corpo{grid-template-columns:1fr}nav{border-right:0;border-bottom:1px solid var(--line)}.regua{grid-template-columns:1fr}main{padding:20px 16px}}
</style></head><body>
<header>${o.catavento}<b>Guia do Novasce OS</b><span class="etq">${esc(o.versao)} · ${esc(o.data)} · Material do time</span></header>
<div class="corpo"><nav><span class="etq">Palavras deste guia</span>${palavras}<span class="etq" style="margin-top:10px">Neste guia</span>${nav}</nav>
<main>${guias.map(secao).join("\n")}
<footer>Algo não bateu? Pedir ajuda, dizendo a seção. Versão publicada não muda: correção vira versão nova.</footer></main></div>
</body></html>`;
  return recusarTravessao(html);
}

export { GUIAS };
