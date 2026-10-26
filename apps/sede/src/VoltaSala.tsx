import { Botao, Catavento } from "@novasce/ui";
import { useEffect, useRef, useState } from "react";

export function VoltaSala({ segundos, voltar, continuar }: { segundos: number; voltar: () => void; continuar: () => void }) {
  const [resta, setResta] = useState(segundos);
  const voltou = useRef(false);
  useEffect(() => {
    setResta(segundos);
    voltou.current = false;
    const t = window.setInterval(() => setResta((r) => Math.max(0, r - 1)), 1000);
    return () => window.clearInterval(t);
  }, [segundos]);
  useEffect(() => {
    if (resta === 0 && !voltou.current) {
      voltou.current = true;
      voltar();
    }
  }, [resta, voltar]);
  return (
    <div className="veu">
      <div className="cartao-volta entra">
        <Catavento tamanho={52} cor="var(--nv-teal)" />
        <h1>Ninguém mexe há 10 minutos</h1>
        <p>Voltando para a Sala em <b className="mono">{resta} s</b>. O que estava aberto no navegador será esquecido.</p>
        <Botao variante="primario" onClick={continuar}>Continuar usando</Botao>
      </div>
    </div>
  );
}
