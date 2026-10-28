import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { EventoDoHelper, Papel } from "./estado";

export type PlacaDeVideo = { nvidia: string | null; varianteDaImagem: "nvidia" | "padrao" };
export type EstadoDaRede = { conectada: boolean; porCabo: boolean };
export type RedeWifi = { ssid: string; sinal: number; protegida: boolean; emUso: boolean };
export type MembroDoTime = { id: string; name: string; email: string; role: string };

/** Tudo que a tela pede ao sistema. Cada função é um comando do src-tauri/src/comandos.rs. */
export const api = {
  placaDeVideo: () => invoke<PlacaDeVideo>("placa_de_video"),
  estadoDaRede: () => invoke<EstadoDaRede>("estado_da_rede"),
  redesWifi: () => invoke<RedeWifi[]>("redes_wifi"),
  conectarWifi: (ssid: string, senha: string) => invoke<void>("conectar_wifi", { ssid, senha }),
  definirPapel: (papel: Papel) => invoke<void>("definir_papel", { papel }),
  registrar: () => invoke<void>("registrar"),
  aoRegistrar: (ouvinte: (evento: EventoDoHelper) => void): Promise<UnlistenFn> =>
    listen<EventoDoHelper>("registro", (evento) => ouvinte(evento.payload)),
  listarTime: () => invoke<MembroDoTime[]>("listar_time"),
  criarConta: (nome: string, email: string, senha: string, dono: string | null) =>
    invoke<string>("criar_conta", { nome, email, senha, dono }),
  prepararSede: () => invoke<void>("preparar_sede"),
  concluir: () => invoke<void>("concluir"),
  reiniciar: () => invoke<void>("reiniciar"),
  fechar: () => invoke<void>("fechar"),
};
