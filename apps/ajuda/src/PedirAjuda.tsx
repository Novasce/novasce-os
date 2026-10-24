import { Botao } from "@novasce/ui";
import { useEffect, useState } from "react";
import * as ponte from "./ponte";
import type { PreparoPedido } from "./tipos";

type Fase = "escrevendo" | "ouvindo" | "enviando" | "guardado";

export function PedirAjuda() {
  const [preparo, setPreparo] = useState<PreparoPedido | null>(null);
  const [texto, setTexto] = useState("");
  const [comPrint, setComPrint] = useState(true);
  const [fase, setFase] = useState<Fase>("escrevendo");
  const [aviso, setAviso] = useState("");

  useEffect(() => {
    void ponte.prepararPedido().then((p) => {
      setPreparo(p);
      setComPrint(Boolean(p.miniatura));
    });
  }, []);

  const enviar = async () => {
    setFase("enviando");
    setAviso("");
    const r = await ponte.enviarPedido(texto, comPrint);
    if (r.tipo === "enviado") return void ponte.fecharJanela("pedir");
    if (r.tipo === "guardado") return setFase("guardado");
    setFase("escrevendo");
    setAviso(r.mensagem ?? "O chamado não foi aberto.");
  };

  const falar = async () => {
    if (fase === "ouvindo") {
      try {
        const t = await ponte.ditarParar();
        setTexto((x) => (x ? `${x} ${t}` : t));
      } catch (e) {
        setAviso(String(e));
      }
      setFase("escrevendo");
      return;
    }
    try {
      await ponte.ditarIniciar();
      setFase("ouvindo");
    } catch (e) {
      setAviso(String(e));
    }
  };

  if (fase === "guardado")
    return (
      <section className="pedir">
        <h1>O chamado não foi enviado</h1>
        <p className="resumo">A internet caiu antes de chegar à Novasce. O pedido ficou guardado neste computador e sai sozinho quando a conexão voltar.</p>
        <div className="acoes-pedir">
          <Botao onClick={async () => { if ((await ponte.tentarAgora()) > 0) void ponte.fecharJanela("pedir"); }}>Tentar agora</Botao>
          <Botao onClick={() => void ponte.fecharJanela("pedir")}>Fechar</Botao>
        </div>
      </section>
    );

  return (
    <section className="pedir">
      <label htmlFor="o-que" className="titulo-pedir">O que aconteceu?</label>
      <div className="caixa-texto">
        <textarea id="o-que" value={texto} onChange={(e) => setTexto(e.target.value)} rows={4} />
        <span className="falar">
          <Botao tamanho="pequeno" onClick={() => void falar()}>{fase === "ouvindo" ? "Parar e escrever" : "Falar"}</Botao>
        </span>
      </div>
      <span className="etq">Vai junto</span>
      <div className="vai-junto">
        <div className="miniatura">{preparo?.miniatura ? <img src={preparo.miniatura} alt="Print desta tela" /> : null}</div>
        <div className="marcas">
          <label><input type="checkbox" checked={comPrint} disabled={!preparo?.miniatura} onChange={(e) => setComPrint(e.target.checked)} />Print desta tela</label>
          <label><input type="checkbox" checked disabled />Dados da máquina: {preparo?.linhaMaquina ?? ""}</label>
          <span style={{ color: "var(--muted)" }}>Nada além disso sai do computador.</span>
        </div>
      </div>
      <span className={`status ${aviso ? "erro" : ""}`} aria-live="polite">{fase === "enviando" ? "Enviando para a Novasce" : fase === "ouvindo" ? "Ouvindo · clique em Parar e escrever" : aviso}</span>
      <div className="acoes-pedir">
        <Botao variante="primario" desligado={!texto.trim() ? "Conte o que aconteceu primeiro." : fase === "enviando" ? "Enviando." : undefined} onClick={() => void enviar()}>Enviar para a Novasce</Botao>
        <Botao onClick={() => void ponte.fecharJanela("pedir")}>Cancelar</Botao>
      </div>
    </section>
  );
}
