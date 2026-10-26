import { fichaPorId } from "@novasce/ajuda";
import { Botao, Ficha } from "@novasce/ui";
import { useState } from "react";
import { SemInternet } from "./Estados";
import * as ponte from "./ponte";
import { StatusCodigo } from "./Status";
import type { ErroVisao, EstadoTela } from "./tipos";
import { usarCodigo } from "./usarCodigo";

export function Passo3({ estado, aoTerminar }: { estado: EstadoTela; aoTerminar: () => void }) {
  const [conectado, setConectado] = useState(Boolean(estado.githubLogin));
  const { f, pedir } = usarCodigo(ponte.githubIniciar, ponte.githubEsperar, () => setConectado(true), estado.githubConfigurado && !estado.githubLogin);
  const [criando, setCriando] = useState(false);
  const [erro, setErro] = useState("");
  const ficha = fichaPorId("github")!;

  if (!estado.githubConfigurado) {
    return (
      <section className="estado-centro">
        <h1>Passo 3 · GitHub e brain</h1>
        <p>O GitHub ainda não foi ligado ao Novasce OS pela Novasce.</p>
        <p>O resto do computador funciona normalmente. Quando a Novasce ligar, o Começar volta neste passo.</p>
        <Botao variante="primario" onClick={() => void ponte.fechar()}>Pular por agora</Botao>
      </section>
    );
  }
  if (f.fase === "erro" && f.erro.tipo === "sem-internet") return <SemInternet passo={3} />;

  const criar = async () => {
    setCriando(true);
    setErro("");
    try {
      await ponte.brainCriar();
      aoTerminar();
    } catch (e) {
      setErro((e as ErroVisao).mensagem);
    } finally {
      setCriando(false);
    }
  };

  const codigo = f.fase === "esperando" || f.fase === "pronto" ? f.codigo : undefined;
  return (
    <>
      <span className="etq teal">Passo 3 · GitHub e brain</span>
      <h1 className="titulo-passo">Seu brain fica guardado no seu GitHub, privado.</h1>
      <div className="grid-2">
        <div className={`cartao ${conectado ? "" : "sel"}`}>
          <b>1. Conectar o GitHub</b>
          <span>Abrimos o GitHub no navegador. Confira o código e aprove. Sem digitar senha aqui.</span>
          <div className="linha-codigo">
            <span className="mono codigo">{codigo?.codigo ?? (conectado ? "" : <span className="esqueleto" style={{ display: "inline-block", width: "9ch", height: 30 }} />)}</span>
            <Botao tamanho="pequeno" desligado={codigo ? undefined : "O código ainda está chegando."} onClick={() => codigo && void ponte.copiar(codigo.codigo)}>Copiar</Botao>
          </div>
          {conectado ? <span className="status ok"><span className="ponto" style={{ background: "var(--ok)" }} />Conectado</span> : <StatusCodigo f={f} pedirOutro={() => void pedir()} />}
          <button type="button" className="link" onClick={() => void ponte.abrirUrl("https://github.com/signup")}>Não tenho conta: criar no GitHub, de graça</button>
        </div>
        <div className={`cartao ${conectado ? "sel" : "apagado"}`}>
          <b style={conectado ? undefined : { color: "var(--muted)" }}>2. Criar o brain</b>
          <span>
            Assim que o GitHub conectar, criamos <span className="mono" style={{ color: "var(--ink)" }}>{estado.githubLogin ? `${estado.githubLogin.toLowerCase()}-brain` : "o seu brain"}</span>, privado, com o kit da Novasce. Ele salva sozinho a cada poucos minutos.
          </span>
          <span>As IAs do passo 2 já trabalham dentro dele.</span>
          <span className={`status ${erro ? "erro" : ""}`}>{criando ? "Criando o seu brain no GitHub" : erro}</span>
          <div style={{ marginTop: "auto" }}>
            <Botao variante="primario" desligado={conectado ? (criando ? "Criando." : undefined) : "Conecte o GitHub primeiro."} onClick={() => void criar()}>Criar meu brain</Botao>
          </div>
        </div>
      </div>
      <Ficha oQueE={ficha.oQueE} paraQueServe={ficha.paraQueServe} comoFunciona={ficha.comoFunciona} quantoCusta={ficha.quantoCusta} precisaInstalar={ficha.precisaInstalar} comando={ficha.comando} />
    </>
  );
}
