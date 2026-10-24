import { useRef, useState } from "react";
import "./ficha.css";

export type FichaProps = { oQueE: string; paraQueServe: string; comoFunciona: string; quantoCusta: string; precisaInstalar: string; comando?: string };

/**
 * Ficha de cinco campos. "Mostrar o comando por trás" troca o conteúdo no mesmo lugar: antes de
 * trocar, a caixa guarda a altura que tinha (só cresce) e a ficha não encolhe nem pula.
 */
export function Ficha({ oQueE, paraQueServe, comoFunciona, quantoCusta, precisaInstalar, comando }: FichaProps) {
  const [verComando, setVer] = useState(false);
  const caixa = useRef<HTMLDivElement>(null);
  const [alturaMinima, setAlturaMinima] = useState(0);
  const alternar = () => {
    setAlturaMinima((a) => Math.max(a, caixa.current?.getBoundingClientRect().height ?? 0));
    setVer(!verComando);
  };
  const campos: [string, string][] = [["O que é", oQueE], ["Para que serve", paraQueServe], ["Como funciona", comoFunciona], ["Quanto custa", quantoCusta], ["Precisa instalar?", precisaInstalar]];
  return (
    <div className="nv-ficha-envolta">
      <div className="nv-ficha ficha" ref={caixa} style={alturaMinima ? { minHeight: alturaMinima } : undefined}>
        {verComando && comando ? <pre className="nv-ficha__comando">{comando}</pre> : campos.map(([r, t]) => (
          <div key={r}><span className="nv-ficha__rotulo">{r}</span><div>{t}</div></div>
        ))}
      </div>
      {comando ? (
        <button type="button" className="nv-ficha__link" onClick={alternar}>{verComando ? "Mostrar a ficha" : "Mostrar o comando por trás"}</button>
      ) : null}
    </div>
  );
}
