/** npm run guia -w ajuda -- --versao v1 --data 23/10/2026 */
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { montarIndice } from "../busca";
import { gerarHtml } from "./html";

const aqui = dirname(fileURLToPath(import.meta.url));
const arg = (nome: string, padrao: string) => {
  const i = process.argv.indexOf(`--${nome}`);
  return i > 0 ? process.argv[i + 1] : padrao;
};
const versao = arg("versao", "v1");
const data = arg("data", new Date().toLocaleDateString("pt-BR"));
const catavento = readFileSync(join(aqui, "../../marca/catavento.svg"), "utf8").replace("<svg ", '<svg aria-hidden="true" ');

const saida = join(aqui, "../saida");
mkdirSync(saida, { recursive: true });
writeFileSync(join(saida, `guia-novasce-os-${versao}.html`), gerarHtml({ versao, data, catavento }));
writeFileSync(join(saida, "indice.json"), JSON.stringify(montarIndice().map(({ texto: _t, ...i }) => i), null, 2));
console.log(`Guia ${versao} e indice.json gravados em ${saida}`);
