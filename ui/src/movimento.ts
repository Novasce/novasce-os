/** Movimento da casa: o fator vem do KDE (AnimationDurationFactor). Zero é "menos movimento". */
export function aplicarMovimento(fator: number, raiz: HTMLElement = document.documentElement): void {
  const f = Number.isFinite(fator) ? Math.min(Math.max(fator, 0), 4) : 1;
  raiz.style.setProperty("--fator", String(f));
  if (f === 0) raiz.dataset.movimento = "reduzido";
  else delete raiz.dataset.movimento;
}

export function movimentoReduzido(raiz: HTMLElement = document.documentElement): boolean {
  return raiz.dataset.movimento === "reduzido" || window.matchMedia?.("(prefers-reduced-motion: reduce)").matches === true;
}
