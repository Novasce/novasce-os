import { AnelProgresso, Botao } from "@novasce/ui";
import { useEffect, useState } from "react";
import * as ponte from "./ponte";
import { NOME_IA, type EstadoTela } from "./tipos";
import { usarCodigo } from "./usarCodigo";

function GithubNaLinha({ aoConectar }: { aoConectar: () => void }) {
  const [ligado, setLigado] = useState(false);
  const { f } = usarCodigo(ponte.githubIniciar, ponte.githubEsperar, async () => { await ponte.brainCriar(); aoConectar(); }, ligado);
  if (!ligado) return <Botao tamanho="pequeno" onClick={() => setLigado(true)}>Conectar o GitHub</Botao>;
  if (f.fase === "esperando") return <span><span className="mono">{f.codigo.codigo}</span> aprove no navegador</span>;
  if (f.fase === "erro") return <span style={{ color: "var(--erro)" }}>{f.erro.mensagem}</span>;
  return <span style={{ color: "var(--muted)" }}>trazendo</span>;
}

export function DeVolta({ estado, recarregar }: { estado: EstadoTela; recarregar: () => Promise<void> }) {
  useEffect(() => {
    const t = window.setInterval(() => void recarregar(), 2000);
    return () => window.clearInterval(t);
  }, [recarregar]);
  const ia = estado.ias[0] ?? "claude";
  const ok = (t: string) => <span style={{ color: "var(--ok)", fontWeight: 600 }}>{t}</span>;
  const alerta = (t: string) => <span style={{ color: "var(--alerta)", fontWeight: 600 }}>{t}</span>;
  const trazidos = estado.espacos.filter((e) => e.estado === "trazido").map((e) => e.nome);
  const docs = estado.sincronizar?.documentos;

  const linhaDocs = !estado.googleConectado ? (
    <Botao tamanho="pequeno" onClick={() => void ponte.googleConectar().then(recarregar)}>Conectar o Google</Botao>
  ) : docs?.estado === "subindo" && docs.progresso !== null ? (
    <span style={{ display: "flex", alignItems: "center", gap: 10 }}><span className="barra"><span style={{ width: `${docs.progresso}%` }} /></span><span className="mono" style={{ color: "var(--ink2)" }}>{docs.progresso}%</span></span>
  ) : docs?.estado === "problema" ? alerta(docs.mensagem ?? "com problema") : ok("de volta");

  return (
    <>
      <div style={{ display: "flex", alignItems: "center", gap: 18 }}>
        <AnelProgresso passos={estado.passosFeitos as 0 | 1 | 2 | 3} tamanho={64} />
        <div>
          <div style={{ font: "700 26px Inter, sans-serif" }}>Que bom ter você de volta, {estado.pessoa?.firstName}.</div>
          <div style={{ fontSize: 15, color: "var(--ink2)", marginTop: 4 }}>Este é um notebook novo. Estamos trazendo o que era seu.</div>
        </div>
      </div>
      <div className="tabela-volta">
        <div><span><b>Meu brain</b> · {estado.brainRepo ?? "o seu brain"}</span>{estado.brainRepo ? ok("de volta") : <GithubNaLinha aoConectar={() => void recarregar()} />}</div>
        <div><span><b>Espaços</b> · {trazidos.join(", ") || "depois do brain"}</span>{trazidos.length ? ok("de volta") : <span style={{ color: "var(--muted)" }}>depois do brain</span>}</div>
        <div><span><b>Documentos</b> · Google Drive</span>{linhaDocs}</div>
        <div><span><b>IA</b> · {NOME_IA[ia]}</span>{alerta("entre de novo")}</div>
        <div><span><b>Senhas</b> · Bitwarden</span>{alerta("entre de novo")}</div>
      </div>
      <div style={{ fontSize: 14, color: "var(--muted)" }}>Pode começar a trabalhar. O resto chega sozinho. Logins de IA e senhas são pedidos de novo de propósito, por segurança.</div>
      <div className="acoes">
        <Botao variante="primario" onClick={() => void ponte.iaAbrirWeb(ia)}>Entrar no {NOME_IA[ia]}</Botao>
        <Botao onClick={async () => { await ponte.deVoltaConcluir(); await ponte.fechar(); }}>Depois</Botao>
      </div>
    </>
  );
}
