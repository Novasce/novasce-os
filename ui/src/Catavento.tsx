/** As três pás do cata-vento oficial (design/novasce/marca-vetor/catavento.svg). */
const PAS = [
  "M1475.296 126.707C1484.370 134.042 1446.451 95.869 1446.451 95.869L1459.001 75.440L1484.865 119.438L1473.158 139.982L1420.156 141.239L1400.138 117.159C1400.138 117.159 1466.222 119.372 1475.296 126.707",
  "M1463.375 149.451C1465.240 137.934 1450.917 189.796 1450.917 189.796L1426.952 189.039L1452.315 144.751L1475.961 144.719L1503.354 190.111L1492.383 219.439C1492.383 219.439 1461.510 160.968 1463.375 149.451",
  "M1490.259 149.515C1479.340 153.627 1531.456 140.252 1531.456 140.252L1542.720 161.417L1491.683 161.447L1479.893 140.950L1505.642 94.607L1536.542 89.534C1536.542 89.534 1501.178 145.403 1490.259 149.515",
] as const;

/** As pás, para usar dentro de outro SVG com viewBox 0 0 142.582 143.999. */
export function PasDoCatavento() {
  return (
    <g transform="translate(-1400.138 219.439) scale(1 -1)">
      {PAS.map((d, i) => (
        <path key={i} d={d} />
      ))}
    </g>
  );
}

export type CataventoProps = { tamanho?: number; cor?: string; titulo?: string };

/** O cata-vento sempre inteiro. A cor vem por `color`, para aceitar variáveis CSS. */
export function Catavento({ tamanho = 24, cor = "currentColor", titulo }: CataventoProps) {
  return (
    <svg
      width={tamanho}
      height={(tamanho * 143.999) / 142.582}
      viewBox="0 0 142.582 143.999"
      fill="currentColor"
      style={{ color: cor }}
      role={titulo ? "img" : undefined}
      aria-label={titulo}
      aria-hidden={titulo ? undefined : true}
    >
      <PasDoCatavento />
    </svg>
  );
}
