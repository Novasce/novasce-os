import { useEffect, useState } from "react";
import { Botao } from "@novasce/ui";
import { api, type PlacaDeVideo } from "../api";
import type { Papel } from "../estado";

export function textoDaPlaca(placa: PlacaDeVideo | null): { texto: string; tom: "ok" | "alerta" | "neutro" } {
  if (!placa) return { texto: "Procurando a placa de vídeo.", tom: "neutro" };
  const { nvidia, varianteDaImagem } = placa;
  if (nvidia && varianteDaImagem === "nvidia") {
    return { texto: `Placa de vídeo encontrada: ${nvidia}. Esta máquina usa a versão com NVIDIA.`, tom: "ok" };
  }
  if (!nvidia && varianteDaImagem === "padrao") {
    return { texto: "Nenhuma placa NVIDIA encontrada. Esta máquina usa a versão padrão.", tom: "ok" };
  }
  if (nvidia) {
    return {
      texto: `Placa de vídeo encontrada: ${nvidia}, mas esta máquina está na versão padrão. Troque para a versão com NVIDIA antes de usar.`,
      tom: "alerta",
    };
  }
  return {
    texto: "Nenhuma placa NVIDIA encontrada, mas esta máquina está na versão com NVIDIA. Troque para a versão padrão antes de usar.",
    tom: "alerta",
  };
}

const CARTOES: { papel: Papel; titulo: string; texto: string }[] = [
  {
    papel: "sede",
    titulo: "Sede",
    texto: "Entra sozinha na conta Sede, abre a Sala em tela cheia e roda os agentes. Contas pessoais entram depois.",
  },
  {
    papel: "notebook",
    titulo: "Notebook",
    texto: "De uma pessoa. Login normal, sem agentes. Você escolhe de quem ele é no passo 4.",
  },
];

export function PassoPapel({ aoDefinir }: { aoDefinir: (papel: Papel) => void }) {
  const [placa, setPlaca] = useState<PlacaDeVideo | null>(null);
  const [escolha, setEscolha] = useState<Papel | null>(null);
  const [ocupado, setOcupado] = useState(false);
  const [erro, setErro] = useState("");

  useEffect(() => {
    api.placaDeVideo().then(setPlaca).catch(() => setPlaca({ nvidia: null, varianteDaImagem: "padrao" }));
  }, []);

  async function continuar() {
    if (!escolha) return;
    setOcupado(true);
    setErro("");
    try {
      await api.definirPapel(escolha);
      aoDefinir(escolha);
    } catch (e) {
      setErro(String(e));
    } finally {
      setOcupado(false);
    }
  }

  const linha = textoDaPlaca(placa);
  return (
    <section className="passo">
      <h1>Qual é o papel desta máquina?</h1>
      <div className="cartoes" role="radiogroup" aria-label="Papel da máquina">
        {CARTOES.map((cartao) => (
          <button
            key={cartao.papel}
            type="button"
            role="radio"
            aria-checked={escolha === cartao.papel}
            aria-label={cartao.titulo}
            className="cartao-papel"
            onClick={() => setEscolha(cartao.papel)}
          >
            <b>{cartao.titulo}</b>
            <span>{cartao.texto}</span>
          </button>
        ))}
      </div>
      <p className={`linha-estado linha-estado--${linha.tom}`}>
        <span className="ponto" aria-hidden="true" />
        {linha.texto}
      </p>
      <p className="aviso-erro" role="alert">{erro}</p>
      <div className="acoes">
        <Botao variante="primario" disabled={!escolha || ocupado} onClick={() => void continuar()}>
          Continuar
        </Botao>
      </div>
    </section>
  );
}
