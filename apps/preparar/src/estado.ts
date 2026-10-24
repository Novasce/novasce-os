/** Os quatro passos do Preparar esta máquina (canvas, prancha 1). */
export type Papel = "sede" | "notebook";
export type Passo = "papel" | "rede" | "registro" | "dono" | "pronto";

/** Uma linha JSON do novasce-helper registrar. */
export type EventoDoHelper =
  | { evento: "codigo"; codigo: string; expiraEm: string }
  | { evento: "aprovado"; maquina: string }
  | { evento: "negado" }
  | { evento: "expirado" }
  | { evento: "sem-modulo" }
  | { evento: "erro"; mensagem: string };

export type Registro =
  | { tipo: "parado" }
  | { tipo: "esperando"; codigo: string }
  | { tipo: "aprovado" }
  | { tipo: "negado" }
  | { tipo: "expirado" }
  | { tipo: "sem-modulo" }
  | { tipo: "erro"; mensagem: string }
  | { tipo: "depois" };

export type Estado = {
  passo: Passo;
  papel: Papel | null;
  rede: string | null;
  registro: Registro;
  contas: string[];
};

export type Acao =
  | { tipo: "papel-definido"; papel: Papel }
  | { tipo: "rede-pronta"; rede: string }
  | { tipo: "registro-iniciado" }
  | { tipo: "evento-do-registro"; evento: EventoDoHelper }
  | { tipo: "registrar-depois" }
  | { tipo: "conta-criada"; login: string }
  | { tipo: "concluir" };

export const estadoInicial: Estado = { passo: "papel", papel: null, rede: null, registro: { tipo: "parado" }, contas: [] };

export function reduzir(estado: Estado, acao: Acao): Estado {
  switch (acao.tipo) {
    case "papel-definido":
      return { ...estado, papel: acao.papel, passo: "rede" };
    case "rede-pronta":
      return { ...estado, rede: acao.rede, passo: "registro" };
    case "registro-iniciado":
      return { ...estado, registro: { tipo: "parado" } };
    case "evento-do-registro":
      return aplicarEvento(estado, acao.evento);
    case "registrar-depois":
      return { ...estado, registro: { tipo: "depois" }, passo: "dono" };
    case "conta-criada": {
      const contas = [...estado.contas, acao.login];
      return { ...estado, contas, passo: estado.papel === "notebook" ? "pronto" : "dono" };
    }
    case "concluir":
      return { ...estado, passo: "pronto" };
  }
}

function aplicarEvento(estado: Estado, evento: EventoDoHelper): Estado {
  switch (evento.evento) {
    case "codigo":
      return { ...estado, registro: { tipo: "esperando", codigo: evento.codigo } };
    case "aprovado":
      return { ...estado, registro: { tipo: "aprovado" }, passo: "dono" };
    case "negado":
      return { ...estado, registro: { tipo: "negado" } };
    case "expirado":
      return { ...estado, registro: { tipo: "expirado" } };
    case "sem-modulo":
      return { ...estado, registro: { tipo: "sem-modulo" } };
    case "erro":
      return { ...estado, registro: { tipo: "erro", mensagem: evento.mensagem } };
  }
}

/** "R7M2PX" vira "R7M 2PX", como no canvas. */
export function formatarCodigo(codigo: string): string {
  return codigo.length === 6 ? `${codigo.slice(0, 3)} ${codigo.slice(3)}` : codigo;
}

export type ItemDaLateral = { passo: Passo; texto: string; situacao: "feito" | "agora" | "falta" };

const ORDEM: Passo[] = ["papel", "rede", "registro", "dono"];
const NOMES: Record<Passo, string> = { papel: "Papel", rede: "Rede", registro: "Registro", dono: "De quem é", pronto: "" };

export function lateral(estado: Estado): ItemDaLateral[] {
  const atual = estado.passo === "pronto" ? ORDEM.length : ORDEM.indexOf(estado.passo);
  const feito: Record<Passo, string> = {
    papel: `Papel: ${estado.papel ?? ""}`,
    rede: `Rede: ${estado.rede ?? ""}`,
    registro: estado.registro.tipo === "aprovado" ? "Registro: aprovado" : "Registro: depois",
    dono: estado.papel === "sede" ? `Contas: ${estado.contas.length}` : `De quem é: ${estado.contas[0] ?? ""}`,
    pronto: "",
  };
  return ORDEM.map((passo, i) =>
    i < atual
      ? { passo, texto: `✓ ${feito[passo]}`, situacao: "feito" }
      : { passo, texto: `${i + 1} · ${NOMES[passo]}`, situacao: i === atual ? "agora" : "falta" },
  );
}
