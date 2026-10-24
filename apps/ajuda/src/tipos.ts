export type TelaInicial = { modo: "central" | "pedir" | "recomendados" | "esqueci" | "detalhe"; tela: string; detalhe: string | null };
export type Novidade = { versao: string; linha: string };
export type PreparoPedido = { linhaMaquina: string; miniatura: string | null };
export type ResultadoPedido = { tipo: "enviado" | "guardado" | "recusado"; numero: string | null; mensagem: string | null };
export type Esqueci = { svg: string; titulo: string; linha: string };
