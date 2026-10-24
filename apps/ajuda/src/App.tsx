import { aplicarMovimento } from "@novasce/ui";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { Central } from "./Central";
import { PedirAjuda } from "./PedirAjuda";
import * as ponte from "./ponte";
import type { TelaInicial } from "./tipos";

export function App() {
  const [inicial, setInicial] = useState<TelaInicial | null>(null);
  const janela = ponte.rotuloJanela();
  useEffect(() => {
    void ponte.movimento().then(aplicarMovimento);
    void ponte.telaInicial().then(setInicial);
    if (import.meta.env.VITE_PONTE === "falsa") return;
    const p = listen<TelaInicial>("ajuda://tela", (e) => setInicial(e.payload));
    return () => void p.then((f) => f());
  }, []);
  if (!inicial) return <div className="esqueleto" style={{ height: "100vh" }} aria-busy="true" />;
  return janela === "pedir" ? <PedirAjuda /> : <Central inicial={inicial} />;
}
