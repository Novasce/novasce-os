import type { FaseCodigo } from "./usarCodigo";

/** Linha de estado com altura reservada: trocar o texto não mexe na tela. */
export function StatusCodigo({ f, pedirOutro }: { f: FaseCodigo; pedirOutro: () => void }) {
  if (f.fase === "pedindo") return <span className="status">Pedindo o código</span>;
  if (f.fase === "esperando")
    return <span className="status alerta"><span className="ponto" style={{ background: "var(--alerta)" }} />Esperando você aprovar no navegador</span>;
  if (f.fase === "pronto") return <span className="status ok"><span className="ponto" style={{ background: "var(--ok)" }} />Conectado</span>;
  return (
    <span className="status erro">
      {f.erro.mensagem}
      <button type="button" className="link" onClick={pedirOutro}>Pedir outro código</button>
    </span>
  );
}
