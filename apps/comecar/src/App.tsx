import { aplicarMovimento } from "@novasce/ui";
import { useCallback, useEffect, useState } from "react";
import { BoasVindasTela } from "./BoasVindas";
import { DeVolta } from "./DeVolta";
import { Espacos } from "./Espacos";
import { Carregando, ErroCarregar } from "./Estados";
import { Fim } from "./Fim";
import { Nav } from "./Nav";
import { Passo1 } from "./Passo1";
import { Passo2 } from "./Passo2";
import { Passo3 } from "./Passo3";
import * as ponte from "./ponte";
import type { EstadoTela } from "./tipos";

export function App() {
  const [estado, setEstado] = useState<EstadoTela | null>(null);
  const [falhou, setFalhou] = useState(false);

  const recarregar = useCallback(async () => {
    try {
      const e = await ponte.estado();
      aplicarMovimento(e.movimento);
      setEstado(e);
      setFalhou(false);
    } catch {
      setFalhou(true);
    }
  }, []);

  useEffect(() => {
    void recarregar();
    const f = () => void recarregar();
    window.addEventListener("focus", f);
    return () => window.removeEventListener("focus", f);
  }, [recarregar]);

  if (falhou && !estado) return <ErroCarregar tentar={() => void recarregar()} />;
  if (!estado) return <Carregando />;

  const semColuna = estado.tela === "boas-vindas" || estado.tela === "fim" || estado.tela === "de-volta";
  const tela = {
    "boas-vindas": <BoasVindasTela estado={estado} aoTerminar={recarregar} />,
    passo1: <Passo1 aoTerminar={recarregar} />,
    passo2: <Passo2 estado={estado} aoTerminar={recarregar} />,
    passo3: <Passo3 estado={estado} aoTerminar={recarregar} />,
    espacos: <Espacos estado={estado} aoTerminar={recarregar} />,
    "de-volta": <DeVolta estado={estado} recarregar={recarregar} />,
    fim: <Fim estado={estado} recarregar={recarregar} />,
  }[estado.tela];

  return (
    <div className="janela-comecar">
      {semColuna ? null : <Nav estado={estado} />}
      <main className="conteudo">{tela}</main>
      <button type="button" className="botao-ajuda" aria-label="Ajuda desta tela" title="Ajuda desta tela" onClick={() => void ponte.abrirAjuda()}>?</button>
    </div>
  );
}
