import { FICHAS_DA_CENTRAL, GUIAS_DA_CENTRAL, buscar, fichaPorId, guiaPorId, telaPorId } from "@novasce/ajuda";
import { Botao } from "@novasce/ui";
import { useEffect, useState } from "react";
import { Detalhe } from "./Detalhe";
import { Esqueci } from "./Esqueci";
import * as ponte from "./ponte";
import { Recomendados } from "./Recomendados";
import type { Novidade, TelaInicial } from "./tipos";

type Vista = { tipo: "inicio" } | { tipo: "detalhe"; alvo: string } | { tipo: "recomendados" } | { tipo: "esqueci" };

function vistaInicial(t: TelaInicial): Vista {
  if (t.modo === "detalhe" && t.detalhe) return { tipo: "detalhe", alvo: t.detalhe };
  if (t.modo === "recomendados") return { tipo: "recomendados" };
  if (t.modo === "esqueci") return { tipo: "esqueci" };
  return { tipo: "inicio" };
}

export function Central({ inicial }: { inicial: TelaInicial }) {
  const [vista, setVista] = useState<Vista>(vistaInicial(inicial));
  const [termo, setTermo] = useState("");
  const [novas, setNovas] = useState<Novidade[]>([]);
  useEffect(() => {
    setVista(vistaInicial(inicial));
  }, [inicial]);
  useEffect(() => {
    void ponte.novidades().then(setNovas).catch(() => setNovas([]));
  }, []);
  const tela = telaPorId(inicial.tela);
  const resultados = buscar(termo);
  const voltar = () => setVista({ tipo: "inicio" });
  const abrir = (alvo: string) => setVista({ tipo: "detalhe", alvo });

  return (
    <div className="central">
      <section className="col col-1">
        <input className="busca" aria-label="Buscar na ajuda" placeholder="Buscar na ajuda" value={termo} onChange={(e) => setTermo(e.target.value)} />
        {termo.trim() ? (
          <div className="lista">
            {resultados.length === 0 ? <p className="resumo">Nada com essas palavras. Tente outra, ou peça ajuda.</p> : null}
            {resultados.map((r) => (
              <button key={`${r.tipo}:${r.id}`} type="button" className="item" onClick={() => abrir(`${r.tipo}:${r.id}`)}>
                <b>{r.titulo}</b>
                <span>{r.trecho}</span>
              </button>
            ))}
          </div>
        ) : (
          <>
            <span className="etq teal">Nesta tela · {tela.nome}</span>
            <h1>{tela.titulo}</h1>
            <p className="resumo">{tela.resumo}</p>
            <div className="lista">
              {tela.perguntas.map((p, n) => (
                <button key={p.titulo} type="button" className="item caixa" onClick={() => abrir(`pergunta:${tela.tela}#${n}`)}>{p.titulo}</button>
              ))}
            </div>
          </>
        )}
      </section>
      {vista.tipo === "inicio" ? (
        <>
          <section className="col col-2">
            <span className="etq">Guias</span>
            <div className="lista linhas">
              {GUIAS_DA_CENTRAL.map((id, n) => (
                <button key={id} type="button" className={`item ${n === 0 ? "forte" : ""}`} onClick={() => abrir(`guia:${id}`)}>{guiaPorId(id)!.titulo}</button>
              ))}
            </div>
          </section>
          <section className="col col-3">
            <span className="etq">Fichas</span>
            <div className="fichas">
              {FICHAS_DA_CENTRAL.map((id) => (
                <Botao key={id} tamanho="pequeno" onClick={() => abrir(`ficha:${id}`)}>{fichaPorId(id)!.nome}</Botao>
              ))}
            </div>
            <span className="etq">Não achou?</span>
            <div><Botao variante="primario" onClick={() => void ponte.abrirPedir()}>Pedir ajuda</Botao></div>
            <span className="etq">Novidades</span>
            <div className="novidades">{novas[0] ? <><b>{novas[0].versao}</b> · {novas[0].linha}</> : "Sem novidades nesta versão."}</div>
            <div className="rodape-col">
              <button type="button" className="link" onClick={() => setVista({ tipo: "recomendados" })}>Recomendados da casa</button>
              <button type="button" className="link" onClick={() => void ponte.abrirComecar()}>Abrir o Começar</button>
            </div>
          </section>
        </>
      ) : (
        <section className="col col-23">
          {vista.tipo === "detalhe" ? <Detalhe alvo={vista.alvo} voltar={voltar} /> : null}
          {vista.tipo === "recomendados" ? <Recomendados voltar={voltar} abrirFicha={(id) => abrir(`ficha:${id}`)} /> : null}
          {vista.tipo === "esqueci" ? <Esqueci voltar={voltar} /> : null}
        </section>
      )}
    </div>
  );
}
