import { Botao } from "@novasce/ui";
import * as ponte from "./ponte";

export function Carregando() {
  return (
    <div className="janela-comecar" aria-busy="true" aria-label="Carregando o Começar">
      <div className="nav-c"><div className="esqueleto" style={{ width: 64, height: 64 }} /><div className="esqueleto" style={{ height: 120 }} /></div>
      <main className="conteudo"><div className="esqueleto" style={{ height: 28, width: "60%" }} /><div className="esqueleto" style={{ height: 180 }} /></main>
    </div>
  );
}

export function SemInternet({ passo }: { passo: number }) {
  return (
    <section className="estado-centro">
      <h1>O passo {passo} precisa de internet</h1>
      <p>Conecte o Wi-Fi e o Começar continua daqui. Nada do que você já fez se perdeu.</p>
      <Botao variante="primario" onClick={() => void ponte.abrirWifi()}>Abrir o Wi-Fi</Botao>
    </section>
  );
}

export function ErroCarregar({ tentar }: { tentar: () => void }) {
  return (
    <section className="estado-centro">
      <h1>O Começar não carregou. Nada do que você já fez se perdeu.</h1>
      <p>O serviço da máquina não respondeu. Tente de novo; se continuar, peça ajuda pelo ponto de interrogação da barra.</p>
      <Botao variante="primario" onClick={tentar}>Tentar de novo</Botao>
    </section>
  );
}
