/**
 * Regra da casa: hover, foco, pressionado, seleção e erro só mudam cor, fundo,
 * contorno e sombra. Devolve cada propriedade que mexeria na geometria.
 */
const PERMITIDAS = new Set([
  "background",
  "background-color",
  "border-color",
  "color",
  "cursor",
  "outline",
  "outline-color",
  "outline-offset",
  "box-shadow",
  "opacity",
]);

const ESTADO = /:(hover|focus|focus-visible|active|disabled)|\[aria-(checked|current|pressed|invalid)/;

export function propriedadesProibidasEmEstados(css: string): string[] {
  const proibidas: string[] = [];
  for (const [, seletor, corpo] of css.matchAll(/([^{}]+)\{([^}]*)\}/g)) {
    if (!ESTADO.test(seletor)) continue;
    for (const declaracao of corpo.split(";")) {
      const nome = declaracao.split(":")[0]?.trim();
      if (nome && !PERMITIDAS.has(nome)) proibidas.push(`${seletor.trim()} → ${nome}`);
    }
  }
  return proibidas;
}
