import "@novasce/ui/tokens.css";
import "@novasce/ui/movimento.css";
import "@novasce/ui/canvas.css";
import "./estilo.css";
import { createRoot } from "react-dom/client";
import { App } from "./App";

createRoot(document.getElementById("raiz")!).render(<App />);
