/** Cores da marca, as mesmas do tokens.css (o teste confere). */
export const cores = {
  teal: "#00666C",
  tealEscuro: "#004D52",
  tealSuave: "#E8F2F2",
  ciano: "#00C4CC",
  escuro: "#0A2528",
  branco: "#F5F5F7",
  linha2: "#CBD3D1",
  ok: "#2E7D4F",
  alerta: "#A85408",
  erro: "#B42318",
  grafite: "#1E1E20",
} as const;

/** Movimento da spec, seção 2: uma curva, troca de estado e o que entra ou sai. */
export const movimento = {
  curva: "cubic-bezier(.2, .7, .2, 1)",
  trocaMs: 160,
  entraMs: 200,
  passoMs: 240,
} as const;
