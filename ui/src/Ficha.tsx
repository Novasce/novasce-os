import { useState } from "react";
import "./ficha.css";

export type FichaProps = { oQueE: string; paraQueServe: string; comoFunciona: string; quantoCusta: string; precisaInstalar: string; comando?: string };

/** Ficha de cinco campos. "Mostrar o comando por trás" troca o conteúdo no mesmo lugar. */
export function Ficha({ oQueE, paraQueServe, comoFunciona, quantoCusta, precisaInstalar, comando }: FichaProps) {
  const [verComando, setVer] = useState(false);
  const campos: [string, string][] = [["O que é", oQueE], ["Para que serve", paraQueServe], ["Como funciona", comoFunciona], ["Quanto custa", quantoCusta], ["Precisa instalar?", precisaInstalar]];
  return (
    <div className="nv-ficha-envolta">
      <div className="nv-ficha ficha">
        {verComando && comando ? <pre className="nv-ficha__comando">{comando}</pre> : campos.map(([r, t]) => (
          <div key={r}><span className="nv-ficha__rotulo">{r}</span><div>{t}</div></div>
        ))}
      </div>
      {comando ? (
        <button type="button" className="nv-ficha__link" onClick={() => setVer(!verComando)}>{verComando ? "Mostrar a ficha" : "Mostrar o comando por trás"}</button>
      ) : null}
    </div>
  );
}
