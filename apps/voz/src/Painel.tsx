export type EstadoVoz = { fase: "oculto" | "segurando" | "maos-livres" | "escrevendo" | "erro"; nivel: number; mensagem: string | null };

const PESOS = [0.5, 0.9, 0.6, 1, 0.4];
const TEXTO: Record<EstadoVoz["fase"], string> = {
  oculto: "",
  segurando: "Ouvindo · solte para escrever",
  "maos-livres": "Ouvindo · toque de novo para escrever",
  escrevendo: "Escrevendo o que você falou",
  erro: "",
};

export function Painel({ estado, reduzido }: { estado: EstadoVoz; reduzido: boolean }) {
  const ouvindo = estado.fase === "segurando" || estado.fase === "maos-livres";
  return (
    <div className="pilula" role="status" aria-live="polite">
      <span className="ponto" style={{ background: ouvindo ? "#FF6B5B" : "#00C4CC" }} />
      <span className="texto">{estado.fase === "erro" ? estado.mensagem : TEXTO[estado.fase]}</span>
      <span className="barras" aria-hidden="true">
        {PESOS.map((p, i) => (
          <i key={i} style={{ height: `${reduzido || !ouvindo ? 10 : Math.round(6 + 16 * Math.min(1, estado.nivel * p * 1.4))}px` }} />
        ))}
      </span>
    </div>
  );
}
