import { fichaPorId } from "@novasce/ajuda";
import { Botao, Campo, Ficha } from "@novasce/ui";
import { useState } from "react";
import * as ponte from "./ponte";
import { NOME_CLI, NOME_IA, type ErroVisao, type EstadoTela } from "./tipos";

type Fase =
  | { tipo: "escolhendo" }
  | { tipo: "instalando"; ia: string; n: number; total: number }
  | { tipo: "feito"; ias: string[] }
  | { tipo: "erro"; mensagem: string };
const IAS = ["claude", "chatgpt", "gemini"];
/** A chave pessoal mora em IA › Novasce AI Tools, aba Chaves (o /config/ia é o gasto da Casa, só do administrador). */
export const CAMINHO_DA_CHAVE = "/ia?tab=chaves";

const lista = (nomes: string[]) => (nomes.length < 2 ? nomes.join("") : `${nomes.slice(0, -1).join(", ")} e ${nomes[nomes.length - 1]}`);

/** O que aparece antes de instalar: o que o botão vai fazer, sem surpresa. */
function oQueVaiAcontecer(marcadas: string[]) {
  if (!marcadas.length) return "Cada pessoa usa a assinatura dela. A Novasce não paga IA.";
  const clis = lista(marcadas.map((ia) => NOME_CLI[ia]));
  return marcadas.length === 1
    ? `Instala o ${clis} e abre o login no navegador.`
    : `Instala uma por vez (${clis}) e abre o login de cada uma no navegador.`;
}

/** Só o Claude e o Gemini ligam ao Tenancy por aqui; liga a primeira marcada. */
function qualLiga(marcadas: string[]) {
  return marcadas.find((x) => x !== "chatgpt");
}

export function Passo2({ estado, aoTerminar }: { estado: EstadoTela; aoTerminar: () => void }) {
  const [marcadas, setMarcadas] = useState<string[]>(estado.ias);
  const [vista, setVista] = useState(estado.ias[0] ?? "claude");
  const [fase, setFase] = useState<Fase>({ tipo: "escolhendo" });
  const [chave, setChave] = useState("");
  const [ligacao, setLigacao] = useState<{ ok: boolean; texto: string } | null>(null);
  const [ligando, setLigando] = useState(false);

  const alternar = (ia: string) => {
    if (fase.tipo === "instalando") return;
    setVista(ia);
    setMarcadas((m) => (m.includes(ia) ? m.filter((x) => x !== ia) : [...m, ia]));
  };

  const instalar = async () => {
    for (const [i, ia] of marcadas.entries()) {
      setFase({ tipo: "instalando", ia, n: i + 1, total: marcadas.length });
      try {
        await ponte.iaInstalar(ia);
        await ponte.iaAbrirWeb(ia);
      } catch (e) {
        setFase({ tipo: "erro", mensagem: (e as ErroVisao).mensagem });
        return;
      }
    }
    setFase({ tipo: "feito", ias: marcadas });
  };

  const iaDaLigacao = qualLiga(fase.tipo === "feito" ? fase.ias : marcadas);
  const ligar = async () => {
    if (!iaDaLigacao) return;
    setLigando(true);
    try {
      await ponte.iaLigarTenancy(iaDaLigacao, chave);
      setLigacao({ ok: true, texto: `Ligada. O ${NOME_IA[iaDaLigacao]} já enxerga o seu Tenancy.` });
    } catch (e) {
      setLigacao({ ok: false, texto: (e as ErroVisao).mensagem });
    } finally {
      setLigando(false);
    }
  };

  const ficha = fichaPorId(vista)!;
  const instalou = fase.tipo === "feito";
  const instalando = fase.tipo === "instalando";
  const motivoLigar = !instalou
    ? "Primeiro instale a IA, no botão Instalar e entrar."
    : !iaDaLigacao
      ? "O Codex ainda não liga ao Tenancy por aqui."
      : !chave.trim()
        ? "Cole a chave primeiro."
        : ligando
          ? "Ligando."
          : undefined;

  return (
    <>
      <span className="etq teal">Passo 2 · IA</span>
      <h1 className="titulo-passo">Qual IA você já assina?</h1>
      <div className="grid-3">
        {IAS.map((ia) => (
          <button key={ia} type="button" className="cartao ia-cartao" aria-pressed={marcadas.includes(ia)} aria-disabled={instalando || undefined} onClick={() => alternar(ia)}>
            <b>{NOME_IA[ia]}</b>
            <span>{fichaPorId(ia)!.paraQueServe}</span>
          </button>
        ))}
      </div>
      <span className={`status ${fase.tipo === "erro" ? "erro" : instalou ? "ok" : instalando ? "alerta" : ""}`} aria-live="polite">
        {instalando && (
          <>
            <span className="ponto pulsa" style={{ background: "var(--alerta)" }} aria-hidden="true" />
            {fase.total > 1 ? `Instalando o ${NOME_CLI[fase.ia]} · ${fase.n} de ${fase.total}` : `Instalando o ${NOME_CLI[fase.ia]}`}
          </>
        )}
        {fase.tipo === "feito" &&
          (fase.ias.length === 1
            ? `Pronto. O login do ${NOME_IA[fase.ias[0]]} abriu no navegador.`
            : `Pronto. Os logins de ${lista(fase.ias.map((ia) => NOME_IA[ia]))} abriram no navegador.`)}
        {fase.tipo === "erro" && fase.mensagem}
        {fase.tipo === "escolhendo" && oQueVaiAcontecer(marcadas)}
      </span>
      <div className="acoes">
        {instalou ? (
          <Botao variante="primario" onClick={async () => { await ponte.iaConcluir(marcadas); aoTerminar(); }}>Continuar</Botao>
        ) : (
          <Botao variante="primario" desligado={marcadas.length ? (instalando ? "Instalando." : undefined) : "Marque pelo menos uma IA."} onClick={() => void instalar()}>
            Instalar e entrar
          </Botao>
        )}
        <Botao desligado={instalando ? "Instalando." : undefined} onClick={async () => { await ponte.iaConcluir([]); aoTerminar(); }}>Ainda não tenho IA</Botao>
      </div>
      <Ficha oQueE={ficha.oQueE} paraQueServe={ficha.paraQueServe} comoFunciona={ficha.comoFunciona} quantoCusta={ficha.quantoCusta} precisaInstalar={ficha.precisaInstalar} comando={ficha.comando} />
      <div className="cartao apagado">
        <b>Depois de instalar: ligar a IA ao Tenancy (opcional)</b>
        <span>
          Ligada, a IA do computador passa a ver o seu quadro no Tenancy: as suas tarefas, os chamados e o seu dia, com o seu acesso.
          {iaDaLigacao ? ` Liga uma IA: o ${NOME_IA[iaDaLigacao]}.` : ""}
          {marcadas.includes("chatgpt") ? " O Codex ainda não liga por aqui." : ""}
        </span>
        <span>A chave é sua: crie no Tenancy, em IA, Novasce AI Tools, aba Chaves, e cole aqui.</span>
        <div className="linha-codigo">
          <Campo rotulo="Chave de IA do Tenancy" value={chave} onChange={(e) => setChave(e.target.value)} placeholder="cole a chave" />
          <Botao desligado={motivoLigar} onClick={() => void ligar()}>Ligar</Botao>
          <button type="button" className="link" onClick={() => void ponte.abrirTenancy(CAMINHO_DA_CHAVE)}>Criar a chave no Tenancy</button>
        </div>
        <span className={`status ${ligacao ? (ligacao.ok ? "ok" : "erro") : ""}`} aria-live="polite">
          {ligacao ? ligacao.texto : motivoLigar && motivoLigar !== "Ligando." ? motivoLigar : ""}
        </span>
      </div>
    </>
  );
}
