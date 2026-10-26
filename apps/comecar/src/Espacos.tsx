import { Botao } from "@novasce/ui";
import { useState } from "react";
import * as ponte from "./ponte";
import type { ErroVisao, EstadoTela } from "./tipos";

export function rotuloTrazer(nomes: string[], kit: boolean): string {
  const itens = nomes.map((n) => `o ${n}`);
  if (kit) itens.push("o kit");
  if (itens.length === 0) return "Continuar";
  if (itens.length === 1) return `Trazer ${itens[0]}`;
  const ultimo = itens.pop();
  return `Trazer ${itens.join(", ")} e ${ultimo}`;
}

const Marca = ({ ligada }: { ligada: boolean }) => (
  <span aria-hidden="true" style={{ width: 20, height: 20, borderRadius: 4, flex: "none", display: "grid", placeItems: "center", fontSize: 11, color: "#fff", background: ligada ? "var(--teal)" : "transparent", border: ligada ? "0" : "1.5px solid var(--line2)", boxSizing: "border-box" }}>{ligada ? "✓" : ""}</span>
);

export function Espacos({ estado, aoTerminar }: { estado: EstadoTela; aoTerminar: () => void }) {
  const podem = estado.espacos.filter((e) => e.estado === "trazer");
  const [marcados, setMarcados] = useState<string[]>(podem.map((e) => e.slug));
  const [kit, setKit] = useState(estado.kitDisponivel);
  const [fase, setFase] = useState<"parado" | "trazendo">("parado");
  const [erro, setErro] = useState("");

  const nomes = podem.filter((e) => marcados.includes(e.slug)).map((e) => e.nome);
  const trazer = async () => {
    setFase("trazendo");
    setErro("");
    try {
      await ponte.espacosTrazer(marcados, kit);
      aoTerminar();
    } catch (e) {
      setErro((e as ErroVisao).mensagem);
    } finally {
      setFase("parado");
    }
  };

  return (
    <>
      <span className="etq teal">Passo 3 · GitHub e brain</span>
      <h1 className="titulo-passo">Seu brain está pronto. Agora, os espaços de que você participa.</h1>
      <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
        <div className="linha-espaco">
          <span aria-hidden="true" style={{ width: 20, height: 20, borderRadius: "50%", background: "var(--ok)", color: "#fff", fontSize: 11, display: "grid", placeItems: "center" }}>{"✓"}</span>
          <div><b>Meu brain</b> <span className="mono" style={{ color: "var(--muted)" }}>{estado.brainRepo}</span><br /><span style={{ color: "var(--muted)" }}>privado, só seu · começa com a Novasce Base</span></div>
        </div>
        {estado.espacos.map((e) =>
          e.estado === "fora" ? (
            <div key={e.slug} className="linha-espaco fora" aria-disabled="true"><Marca ligada={false} /><div><b>{e.nome}</b><br />{e.descricao}</div></div>
          ) : (
            <button
              key={e.slug}
              type="button"
              className={`linha-espaco ${marcados.includes(e.slug) || e.estado === "trazido" ? "sel" : ""}`}
              aria-pressed={marcados.includes(e.slug) || e.estado === "trazido"}
              disabled={e.estado === "trazido"}
              onClick={() => setMarcados((m) => (m.includes(e.slug) ? m.filter((x) => x !== e.slug) : [...m, e.slug]))}
            >
              <Marca ligada={marcados.includes(e.slug) || e.estado === "trazido"} />
              <div style={{ textAlign: "left" }}><b>{e.nome}</b><br /><span style={{ color: "var(--ink2)" }}>{e.estado === "trazido" ? "já está neste computador" : e.descricao}</span></div>
            </button>
          ),
        )}
        {estado.kitDisponivel ? (
          <button type="button" className={`linha-espaco ${kit ? "sel" : ""}`} aria-pressed={kit} onClick={() => setKit(!kit)}>
            <Marca ligada={kit} />
            <div style={{ textAlign: "left" }}><b>Kit da sua frente</b><br /><span style={{ color: "var(--ink2)" }}>chega pelo Tenancy, só para quem tem uma frente lá</span></div>
          </button>
        ) : (
          <div className="linha-espaco" aria-disabled="true"><Marca ligada={false} /><div><b>Kit da sua frente</b><br /><span style={{ color: "var(--ink2)" }}>chega pelo Tenancy quando você tiver uma frente lá</span></div></div>
        )}
      </div>
      <div className="nota">Um espaço nunca lê o seu brain. Um espaço não lê o outro. O que passa de um para outro vai como pacote revisado.</div>
      <span className={`status ${erro ? "erro" : ""}`}>{fase === "trazendo" ? "Trazendo os espaços do GitHub" : erro}</span>
      <div className="acoes">
        <Botao variante="primario" desligado={fase === "trazendo" ? "Trazendo." : undefined} onClick={() => void trazer()}>{rotuloTrazer(nomes, kit)}</Botao>
        <Botao onClick={async () => { await ponte.espacosAgoraNao(); aoTerminar(); }}>Agora não</Botao>
      </div>
    </>
  );
}
