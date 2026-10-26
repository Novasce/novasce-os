import { PasDoCatavento } from "./Catavento";
import "./anel.css";

export type AnelProgressoProps = { passos: 0 | 1 | 2 | 3; tamanho?: number; rotulo?: string };

const RAIO = 44;
const VOLTA = 2 * Math.PI * RAIO;
const TERCO = VOLTA / 3;
const VAO = 6;

/** Progresso em três terços pelo contorno; o cata-vento no meio fica sempre inteiro. */
export function AnelProgresso({ passos, tamanho = 92, rotulo }: AnelProgressoProps) {
  return (
    <svg
      className="nv-anel"
      width={tamanho}
      height={tamanho}
      viewBox="0 0 100 100"
      role="img"
      aria-label={rotulo ?? `${passos} de 3 passos`}
    >
      {[0, 1, 2].map((i) => (
        <circle
          key={i}
          className="nv-anel__terco"
          data-feito={i < passos ? "sim" : "nao"}
          cx="50"
          cy="50"
          r={RAIO}
          fill="none"
          strokeWidth="5"
          strokeDasharray={`${TERCO - VAO} ${VOLTA - TERCO + VAO}`}
          strokeDashoffset={-i * TERCO}
          transform="rotate(-90 50 50)"
          style={{ stroke: i < passos ? "var(--nv-teal)" : "var(--nv-linha-2)" }}
        />
      ))}
      <svg x="25" y="25" width="50" height="50" viewBox="0 0 142.582 143.999" style={{ color: "var(--nv-teal)" }} fill="currentColor">
        <PasDoCatavento />
      </svg>
    </svg>
  );
}
