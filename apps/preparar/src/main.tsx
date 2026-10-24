import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@novasce/ui/tokens.css";
import "./estilo.css";
import { App } from "./App";

const raiz = document.getElementById("raiz");
if (!raiz) throw new Error("Falta o #raiz no index.html");
createRoot(raiz).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
