import { useEffect, useState } from "react";
import { Botao, Campo } from "@novasce/ui";
import { api, type MembroDoTime } from "../api";
import type { Acao, Papel } from "../estado";

type Props = { papel: Papel; registrado: boolean; contas: string[]; despachar: (acao: Acao) => void };

export const SENHA_MINIMA = 8;

/** A senha da pessoa e a repetição: pronta quando tem o mínimo e as duas batem. */
export function lerSenha(senha: string, repetida: string): { pronta: boolean; aviso: string } {
  const naoBatem = repetida !== "" && repetida !== senha;
  return {
    pronta: senha.length >= SENHA_MINIMA && senha === repetida,
    aviso: naoBatem ? "As duas senhas não batem." : "",
  };
}

export function PassoDono({ papel, registrado, contas, despachar }: Props) {
  const [time, setTime] = useState<MembroDoTime[] | null>(registrado ? null : []);
  const [avisoDoTime, setAvisoDoTime] = useState("");
  const [escolhido, setEscolhido] = useState<string | null>(null);
  const [nome, setNome] = useState("");
  const [email, setEmail] = useState("");
  const [senha, setSenha] = useState("");
  const [repetida, setRepetida] = useState("");
  const [ocupado, setOcupado] = useState(false);
  const [erro, setErro] = useState("");

  useEffect(() => {
    if (!registrado) return;
    api
      .listarTime()
      .then(setTime)
      .catch((e) => {
        setAvisoDoTime(String(e));
        setTime([]);
      });
  }, [registrado]);

  const carregando = registrado && time === null;
  const lista = time ?? [];
  const pelaLista = registrado && lista.length > 0;
  const pessoa = pelaLista ? (lista.find((p) => p.id === escolhido) ?? null) : null;
  const leituraDaSenha = lerSenha(senha, repetida);
  const pessoaPronta = carregando ? false : pelaLista ? pessoa !== null : nome.trim() !== "" && email.includes("@");
  const pronto = pessoaPronta && leituraDaSenha.pronta;

  async function criar() {
    const alvo = pessoa
      ? { nome: pessoa.name, email: pessoa.email, dono: papel === "notebook" ? pessoa.id : null }
      : { nome: nome.trim(), email: email.trim(), dono: null };
    setOcupado(true);
    setErro("");
    try {
      const login = await api.criarConta(alvo.nome, alvo.email, senha, alvo.dono);
      setEscolhido(null);
      setNome("");
      setEmail("");
      setSenha("");
      setRepetida("");
      despachar({ tipo: "conta-criada", login });
    } catch (e) {
      setErro(String(e));
    } finally {
      setOcupado(false);
    }
  }

  async function concluirSede() {
    setOcupado(true);
    setErro("");
    try {
      await api.prepararSede();
      despachar({ tipo: "concluir" });
    } catch (e) {
      setErro(String(e));
    } finally {
      setOcupado(false);
    }
  }

  const titulo = papel === "sede" ? "Contas pessoais da sede" : "De quem é este notebook?";
  const explicacao = pelaLista
    ? "Escolha a pessoa na lista do time do Tenancy e peça para ela digitar a senha dela. A conta nasce com o nome dela e já entra com essa senha."
    : "Digite o nome e o e-mail da Novasce da pessoa e peça para ela digitar a senha dela. A conta já entra com essa senha.";

  return (
    <section className="passo">
      <h1>{titulo}</h1>
      <p className="texto">{explicacao}</p>
      {avisoDoTime && (
        <p className="linha-estado linha-estado--alerta">
          <span className="ponto" aria-hidden="true" />
          {avisoDoTime}
        </p>
      )}
      {carregando ? (
        <p className="texto texto--apagado">Buscando a lista do time.</p>
      ) : pelaLista ? (
        <div className="time" role="radiogroup" aria-label="Pessoa do time">
          {lista.map((p) => (
            <button
              key={p.id}
              type="button"
              role="radio"
              aria-checked={escolhido === p.id}
              aria-label={`${p.name}, ${p.email}`}
              className="cartao-escolha"
              onClick={() => setEscolhido(p.id)}
            >
              <b>{p.name}</b>
              <span>{p.email}</span>
            </button>
          ))}
        </div>
      ) : (
        <div className="campos">
          <Campo rotulo="Nome completo" value={nome} onChange={(e) => setNome(e.target.value)} autoComplete="off" />
          <Campo rotulo="E-mail da Novasce" type="email" value={email} onChange={(e) => setEmail(e.target.value)} autoComplete="off" />
        </div>
      )}
      <div className="campos">
        <Campo
          rotulo="Senha da pessoa"
          type="password"
          value={senha}
          onChange={(e) => setSenha(e.target.value)}
          autoComplete="new-password"
          mensagem={`Pelo menos ${SENHA_MINIMA} caracteres. Só a pessoa sabe.`}
        />
        <Campo
          rotulo="Repita a senha"
          type="password"
          value={repetida}
          onChange={(e) => setRepetida(e.target.value)}
          autoComplete="new-password"
          mensagem={leituraDaSenha.aviso}
          erro={leituraDaSenha.aviso !== ""}
        />
      </div>
      {papel === "sede" && (
        <p className="texto texto--apagado">
          {contas.length === 0 ? "Nenhuma conta pessoal criada ainda." : `Contas criadas: ${contas.join(", ")}.`}
        </p>
      )}
      <p className="aviso-erro" role="alert">{erro}</p>
      <div className="acoes">
        <Botao variante="primario" disabled={!pronto || ocupado} onClick={() => void criar()}>
          Criar a conta
        </Botao>
        {papel === "sede" && (
          <Botao disabled={ocupado} onClick={() => void concluirSede()}>
            Concluir a sede
          </Botao>
        )}
      </div>
    </section>
  );
}
