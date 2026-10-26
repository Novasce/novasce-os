import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import * as falsa from "./ponte-falsa";
import type { Esqueci, Novidade, PreparoPedido, ResultadoPedido, TelaInicial } from "./tipos";

const f = import.meta.env.VITE_PONTE === "falsa";
const i = <T>(c: string, a?: Record<string, unknown>) => invoke<T>(c, a);

export const rotuloJanela = (): string => (f ? falsa.rotuloJanela() : getCurrentWindow().label);
export const telaInicial = (): Promise<TelaInicial> => (f ? falsa.telaInicial() : i("tela_inicial"));
export const prepararPedido = (): Promise<PreparoPedido> => (f ? falsa.prepararPedido() : i("preparar_pedido"));
export const enviarPedido = (texto: string, comPrint: boolean): Promise<ResultadoPedido> => (f ? falsa.enviarPedido() : i("enviar_pedido", { texto, comPrint }));
export const tentarAgora = (): Promise<number> => (f ? falsa.zero() : i("tentar_agora"));
export const ditarIniciar = (): Promise<void> => (f ? falsa.nada() : i("ditar_iniciar"));
export const ditarNivel = (): Promise<number> => (f ? falsa.zero() : i("ditar_nivel"));
export const ditarParar = (): Promise<string> => (f ? falsa.ditado() : i("ditar_parar"));
export const recomendadosInstalados = (ids: string[]): Promise<string[]> => (f ? falsa.instalados() : i("recomendados_instalados", { ids }));
export const instalarRecomendado = (id: string): Promise<void> => (f ? falsa.nada() : i("instalar_recomendado", { id }));
export const esqueci = (): Promise<Esqueci> => (f ? falsa.esqueci() : i("esqueci"));
export const novidades = (): Promise<Novidade[]> => (f ? falsa.novidades() : i("novidades"));
export const abrirPedir = (): Promise<void> => (f ? falsa.nada() : i("abrir_pedir"));
export const abrirComecar = (): Promise<void> => (f ? falsa.nada() : i("abrir_comecar"));
export const abrirUrl = (url: string): Promise<void> => (f ? falsa.nada() : i("abrir_url", { url }));
export const fecharJanela = (rotulo: string): Promise<void> => (f ? falsa.nada() : i("fechar_janela", { rotulo }));
export const movimento = (): Promise<number> => (f ? falsa.um() : i("movimento"));
