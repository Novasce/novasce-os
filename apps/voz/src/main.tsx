import "@novasce/ui/tokens.css";
import "@novasce/ui/movimento.css";
import "./estilo.css";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { aplicarMovimento, movimentoReduzido } from "@novasce/ui";
import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { Painel, type EstadoVoz } from "./Painel";

function App() {
  const [estado, setEstado] = useState<EstadoVoz>({ fase: "oculto", nivel: 0, mensagem: null });
  useEffect(() => {
    void invoke<number>("movimento").then(aplicarMovimento);
    const p = listen<EstadoVoz>("voz://estado", (e) => setEstado(e.payload));
    return () => void p.then((f) => f());
  }, []);
  return <Painel estado={estado} reduzido={movimentoReduzido()} />;
}

createRoot(document.getElementById("raiz")!).render(<App />);
