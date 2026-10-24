import { Botao } from "@novasce/ui";
import { useEffect, useState } from "react";
import * as ponte from "./ponte";
import type { Esqueci as Dados } from "./tipos";

export function Esqueci({ voltar }: { voltar: () => void }) {
  const [d, setD] = useState<Dados | null>(null);
  const [erro, setErro] = useState("");
  useEffect(() => {
    void ponte.esqueci().then(setD).catch((e) => setErro(String(e)));
  }, []);
  return (
    <section className="esqueci">
      <div className="qr" aria-label="Código para o celular" dangerouslySetInnerHTML={{ __html: d?.svg ?? "" }} />
      <div className="esqueci-texto">
        <h2>{d?.titulo ?? (erro ? "Não deu para gerar o código" : "Esqueceu a senha?")}</h2>
        <p className="resumo">{erro || d?.linha || ""}</p>
        <p style={{ fontSize: 14, color: "var(--muted)", margin: 0 }}>Sem celular? Fale com o Eduardo.</p>
        <div style={{ marginTop: "auto" }}><Botao onClick={voltar}>Voltar</Botao></div>
      </div>
    </section>
  );
}
