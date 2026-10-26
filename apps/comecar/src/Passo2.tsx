import { fichaPorId } from "@novasce/ajuda";
import { Botao, Campo, Ficha } from "@novasce/ui";
import { useState } from "react";
import * as ponte from "./ponte";
import { NOME_CLI, NOME_IA, type ErroVisao, type EstadoTela } from "./tipos";

type Fase = { tipo: "escolhendo" } | { tipo: "instalando"; ia: string } | { tipo: "feito"; ia: string } | { tipo: "erro"; mensagem: string };
const IAS = ["claude", "chatgpt", "gemini"];

export function Passo2({ estado, aoTerminar }: { estado: EstadoTela; aoTerminar: () => void }) {
  const [marcadas, setMarcadas] = useState<string[]>(estado.ias);
  const [vista, setVista] = useState(estado.ias[0] ?? "claude");
  const [fase, setFase] = useState<Fase>({ tipo: "escolhendo" });
  const [chave, setChave] = useState("");
  const [ligacao, setLigacao] = useState<string>("");

  const alternar = (ia: string) => {
    setVista(ia);
    setMarcadas((m) => (m.includes(ia) ? m.filter((x) => x !== ia) : [...m, ia]));
  };

  const instalar = async () => {
    for (const ia of marcadas) {
      setFase({ tipo: "instalando", ia });
      try {
        await ponte.iaInstalar(ia);
        await ponte.iaAbrirWeb(ia);
      } catch (e) {
        setFase({ tipo: "erro", mensagem: (e as ErroVisao).mensagem });
        return;
      }
    }
    setFase({ tipo: "feito", ia: marcadas[0] });
  };

  const ligar = async () => {
    const ia = marcadas.find((x) => x !== "chatgpt") ?? marcadas[0];
    try {
      await ponte.iaLigarTenancy(ia, chave);
      setLigacao(`Ligada. O ${NOME_IA[ia]} já enxerga o seu Tenancy.`);
    } catch (e) {
      setLigacao((e as ErroVisao).mensagem);
    }
  };

  const ficha = fichaPorId(vista)!;
  const instalou = fase.tipo === "feito";
  return (
    <>
      <span className="etq teal">Passo 2 · IA</span>
      <h1 className="titulo-passo">Qual IA você já assina?</h1>
      <div className="grid-3">
        {IAS.map((ia) => (
          <button key={ia} type="button" className="cartao ia-cartao" aria-pressed={marcadas.includes(ia)} onClick={() => alternar(ia)}>
            <b>{NOME_IA[ia]}</b>
            <span>{fichaPorId(ia)!.paraQueServe}</span>
          </button>
        ))}
      </div>
      <span className={`status ${fase.tipo === "erro" ? "erro" : fase.tipo === "feito" ? "ok" : ""}`} aria-live="polite">
        {fase.tipo === "instalando" && `Instalando o ${NOME_CLI[fase.ia]}`}
        {fase.tipo === "feito" && `Pronto. O login do ${NOME_IA[fase.ia]} abriu no navegador.`}
        {fase.tipo === "erro" && fase.mensagem}
        {fase.tipo === "escolhendo" && "Cada pessoa usa a assinatura dela. A Novasce não paga IA."}
      </span>
      <Ficha oQueE={ficha.oQueE} paraQueServe={ficha.paraQueServe} comoFunciona={ficha.comoFunciona} quantoCusta={ficha.quantoCusta} precisaInstalar={ficha.precisaInstalar} comando={ficha.comando} />
      <div className="cartao apagado">
        <b>Ligar a IA ao Tenancy (opcional)</b>
        <span>Crie a sua chave em Configurações, IA, no Tenancy, e cole aqui. A IA passa a ver o seu quadro.</span>
        <div className="linha-codigo">
          <Campo rotulo="Chave de IA do Tenancy" value={chave} onChange={(e) => setChave(e.target.value)} placeholder="cole a chave" />
          <Botao desligado={!instalou ? "Instale a IA primeiro." : chave.trim() ? undefined : "Cole a chave primeiro."} onClick={() => void ligar()}>Ligar</Botao>
          <button type="button" className="link" onClick={() => void ponte.abrirTenancy("/config/ia")}>Criar a chave no Tenancy</button>
        </div>
        <span className="status">{ligacao}</span>
      </div>
      <div className="acoes">
        {instalou ? (
          <Botao variante="primario" onClick={async () => { await ponte.iaConcluir(marcadas); aoTerminar(); }}>Continuar</Botao>
        ) : (
          <Botao variante="primario" desligado={marcadas.length ? (fase.tipo === "instalando" ? "Instalando." : undefined) : "Marque pelo menos uma IA."} onClick={() => void instalar()}>
            Instalar e entrar
          </Botao>
        )}
        <Botao onClick={async () => { await ponte.iaConcluir([]); aoTerminar(); }}>Ainda não tenho IA</Botao>
      </div>
    </>
  );
}
