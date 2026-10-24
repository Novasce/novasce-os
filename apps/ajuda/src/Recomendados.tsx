import { RECOMENDADOS } from "@novasce/ajuda";
import { Botao } from "@novasce/ui";
import { useEffect, useState } from "react";
import * as ponte from "./ponte";

type Situacao = "disponivel" | "instalando" | "instalado" | "erro";

export function Recomendados({ voltar, abrirFicha }: { voltar: () => void; abrirFicha: (id: string) => void }) {
  const [sit, setSit] = useState<Record<string, Situacao>>({});
  const [erros, setErros] = useState<Record<string, string>>({});
  useEffect(() => {
    void ponte.recomendadosInstalados(RECOMENDADOS.map((r) => r.flatpak)).then((ja) => {
      setSit(Object.fromEntries(RECOMENDADOS.map((r) => [r.flatpak, ja.includes(r.flatpak) ? "instalado" : "disponivel"])));
    });
  }, []);
  const instalar = async (id: string) => {
    setSit((s) => ({ ...s, [id]: "instalando" }));
    try {
      await ponte.instalarRecomendado(id);
      setSit((s) => ({ ...s, [id]: "instalado" }));
      setErros((e) => ({ ...e, [id]: "Instalado só para você." }));
    } catch (e) {
      setSit((s) => ({ ...s, [id]: "erro" }));
      setErros((x) => ({ ...x, [id]: String(e) }));
    }
  };
  return (
    <section className="detalhe">
      <Botao tamanho="pequeno" onClick={voltar}>Voltar</Botao>
      <h2>Recomendados da casa</h2>
      <p className="resumo">Instalar não pede senha de administrador: o app entra só para você.</p>
      <div className="grade-recomendados">
        {RECOMENDADOS.map((r) => {
          const s = sit[r.flatpak] ?? "disponivel";
          return (
            <div key={r.id} className="cartao">
              <b>{r.nome}</b>
              <span>{r.descricao}</span>
              <span className={`status ${s === "erro" ? "erro" : s === "instalado" ? "ok" : ""}`}>{s === "instalando" ? `Instalando o ${r.nome}` : erros[r.flatpak] ?? ""}</span>
              <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                {s === "instalado" ? (
                  <Botao tamanho="pequeno" desligado="Já está neste computador.">Já instalado</Botao>
                ) : (
                  <Botao tamanho="pequeno" variante="primario" desligado={s === "instalando" ? "Instalando." : undefined} onClick={() => void instalar(r.flatpak)}>Instalar</Botao>
                )}
                <button type="button" className="link" onClick={() => abrirFicha(r.ficha)}>Ficha</button>
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}
