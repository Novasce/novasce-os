import "@novasce/ui/tokens.css";
import "@novasce/ui/movimento.css";
import "@novasce/ui/canvas.css";
import "./estilo.css";
import { aplicarMovimento } from "@novasce/ui";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { Canto } from "./Canto";
import { VoltaSala } from "./VoltaSala";

type CantoDados = { versao: string; motivo: string | null };

function App() {
  const janela = getCurrentWindow().label;
  const [volta, setVolta] = useState<number | null>(null);
  const [canto, setCanto] = useState<CantoDados | null>(null);
  useEffect(() => {
    void invoke<number>("movimento").then(aplicarMovimento);
    const a = listen<number | null>("sede://volta", (e) => setVolta(e.payload));
    const b = listen<CantoDados | null>("sede://canto", (e) => setCanto(e.payload));
    return () => { void a.then((f) => f()); void b.then((f) => f()); };
  }, []);
  if (janela === "volta" && volta !== null) return <VoltaSala segundos={volta} voltar={() => void invoke("voltar_agora")} continuar={() => void invoke("continuar_usando")} />;
  if (janela === "canto" && canto) return <Canto versao={canto.versao} motivo={canto.motivo} reiniciar={() => void invoke("reiniciar")} />;
  return null;
}

createRoot(document.getElementById("raiz")!).render(<App />);
