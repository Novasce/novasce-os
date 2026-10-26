import type { Esqueci, Novidade, PreparoPedido, ResultadoPedido, TelaInicial } from "./tipos";

const q = new URLSearchParams(globalThis.location?.search ?? "");
export const nada = async () => undefined;
export const zero = async () => 0;
export const um = async () => 1;
export const rotuloJanela = () => q.get("janela") ?? "central";
export const telaInicial = async (): Promise<TelaInicial> => ({ modo: (q.get("modo") as TelaInicial["modo"]) ?? "central", tela: q.get("tela") ?? "comecar", detalhe: null });
export const prepararPedido = async (): Promise<PreparoPedido> => ({ linhaMaquina: "Novasce OS 1.0.3 · notebook de exemplo · atualizado há 2 dias · disco cifrado", miniatura: null });
export const enviarPedido = async (): Promise<ResultadoPedido> => ({ tipo: q.get("resultado") === "guardado" ? "guardado" : "recusado", numero: null, mensagem: "O chamado não foi aberto. Exemplo de erro." });
export const ditado = async () => "texto ditado de exemplo";
export const instalados = async () => ["org.libreoffice.LibreOffice"];
export const esqueci = async (): Promise<Esqueci> => ({ svg: '<svg xmlns="http://www.w3.org/2000/svg" width="156" height="156"></svg>', titulo: "Esqueceu a senha?", linha: "Exemplo · dados fictícios." });
export const novidades = async (): Promise<Novidade[]> => [{ versao: "1.0.4", linha: "Exemplo de novidade." }];
