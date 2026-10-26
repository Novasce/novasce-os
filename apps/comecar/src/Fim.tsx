import { AnelProgresso, Botao } from "@novasce/ui";
import { useEffect, useRef, useState } from "react";
import * as ponte from "./ponte";
import type { ErroVisao, EstadoTela } from "./tipos";

export function Fim({ estado, recarregar }: { estado: EstadoTela; recarregar: () => Promise<void> }) {
  const visto = useRef(false);
  const [manha, setManha] = useState(estado.manha);
  const [google, setGoogle] = useState<"parado" | "esperando" | "erro">("parado");
  const [erroGoogle, setErroGoogle] = useState("");

  useEffect(() => {
    if (!visto.current) {
      visto.current = true;
      void ponte.fimVisto();
    }
  }, []);

  const conectar = async () => {
    setGoogle("esperando");
    try {
      await ponte.googleConectar();
      setGoogle("parado");
      await recarregar();
    } catch (e) {
      setGoogle("erro");
      setErroGoogle((e as ErroVisao).mensagem);
    }
  };

  const d = estado.descobertas;
  const descoberta = (feito: boolean, texto: string) => (
    <span style={{ color: feito ? "var(--ok)" : undefined }}>{feito ? "✓" : "○"} {texto}</span>
  );

  return (
    <section style={{ display: "flex", flexDirection: "column", gap: 18, flex: 1 }}>
      <div style={{ display: "flex", alignItems: "center", gap: 22 }}>
        <AnelProgresso passos={3} tamanho={92} />
        <div>
          <div style={{ font: "700 30px/1.15 Inter, sans-serif" }}>Pronto. Os três passos estão feitos.</div>
          <div style={{ fontSize: 16, color: "var(--ink2)", marginTop: 6 }}>Tenancy, IA e brain funcionando. O resto é opcional e leva um minuto cada.</div>
        </div>
      </div>
      <span className="etq">Deixe o dia ainda mais fácil</span>
      <div className="grid-3">
        <div className="cartao">
          <b>Arquivos na nuvem</b>
          <span>A pasta Documentos fica salva no seu Google Drive. Se o notebook sumir, nada se perde.</span>
          <span className={`status ${google === "erro" ? "erro" : estado.googleConectado ? "ok" : ""}`}>
            {estado.googleConectado ? "Conectado" : google === "esperando" ? "Esperando você entrar no Google" : google === "erro" ? erroGoogle : ""}
          </span>
          <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <Botao variante="primario" desligado={estado.googleConectado ? "Já está conectado." : google === "esperando" ? "Esperando o Google." : undefined} onClick={() => void conectar()}>Conectar o Google</Botao>
            <button type="button" className="link" onClick={() => void ponte.abrirFicha("drive")}>Ficha</button>
          </div>
        </div>
        <div className="cartao">
          <b>Senhas</b>
          <span>As suas no Bitwarden, de graça. As do time no Cofre do Tenancy, que já está ligado.</span>
          <span className="status" />
          <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <Botao onClick={() => void ponte.abrirBitwarden()}>Abrir o Bitwarden</Botao>
            <button type="button" className="link" onClick={() => void ponte.abrirFicha("bitwarden")}>Ficha</button>
          </div>
        </div>
        <div className="cartao">
          <b>Falar em vez de digitar</b>
          <span>{estado.vozQualquerCampo ? "Segure Super + Espaço e fale. Roda no computador, sem internet e sem custo. O atalho é trocável." : "Use o botão Falar nos apps da casa. Roda no computador, sem internet e sem custo."}</span>
          <span className="status" />
          <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <Botao onClick={() => void ponte.vozTestar()}>Testar agora</Botao>
            <button type="button" className="link" onClick={() => void ponte.abrirFicha("voz")}>Ficha</button>
          </div>
        </div>
      </div>
      <label className="linha-espaco" style={{ cursor: "pointer" }}>
        <input type="checkbox" checked={manha} onChange={(e) => { setManha(e.target.checked); void ponte.manhaDefinir(e.target.checked); }} />
        <b>Abrir o Meu dia ao chegar de manhã</b>
        <span style={{ color: "var(--muted)" }}>dá para desligar quando quiser</span>
      </label>
      <div className="descobertas">
        <span className="etq">Descobertas</span>
        {descoberta(d.falou, "Falou pela primeira vez")}
        {descoberta(d.pediuAjuda, "Pediu ajuda uma vez")}
        {descoberta(d.achouNota, "Achou uma nota pela busca")}
        {descoberta(d.trocouAtalho, "Trocou o atalho de falar")}
        <span style={{ color: "var(--muted)" }}>sem pontos e sem sequência de dias</span>
      </div>
      <div style={{ marginTop: "auto", fontSize: 13, color: "var(--muted)" }}>Cada um tem a ficha de cinco campos. O Começar some quando tudo está feito e volta pela Ajuda.</div>
    </section>
  );
}
