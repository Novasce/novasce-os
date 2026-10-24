import { useEffect, useState } from "react";
import { Botao, Campo } from "@novasce/ui";
import { api, type EstadoDaRede, type RedeWifi } from "../api";

export function PassoRede({ aoConectar }: { aoConectar: (rede: string) => void }) {
  const [rede, setRede] = useState<EstadoDaRede | null>(null);
  const [redes, setRedes] = useState<RedeWifi[]>([]);
  const [escolhida, setEscolhida] = useState<RedeWifi | null>(null);
  const [senha, setSenha] = useState("");
  const [ocupado, setOcupado] = useState(false);
  const [erro, setErro] = useState("");

  useEffect(() => {
    api.estadoDaRede().then(setRede).catch(() => setRede({ conectada: false, porCabo: false }));
    api.redesWifi().then(setRedes).catch(() => setRedes([]));
  }, []);

  const emUso = redes.find((r) => r.emUso)?.ssid;
  const nomeDaRede = rede?.porCabo ? "cabo" : (emUso ?? "internet");

  async function conectar() {
    if (!escolhida) return;
    setOcupado(true);
    setErro("");
    try {
      await api.conectarWifi(escolhida.ssid, senha);
      setSenha("");
      aoConectar(escolhida.ssid);
    } catch (e) {
      setErro(String(e));
    } finally {
      setOcupado(false);
    }
  }

  const situacao =
    rede === null
      ? "Conferindo a internet."
      : rede.conectada
        ? rede.porCabo
          ? "Esta máquina já está na internet, pelo cabo."
          : `Esta máquina já está na internet, pela rede ${nomeDaRede}.`
        : "Escolha a rede Wi-Fi e digite a senha.";

  return (
    <section className="passo">
      <h1>Conectar à internet</h1>
      <p className={`linha-estado ${rede?.conectada ? "linha-estado--ok" : "linha-estado--neutro"}`}>
        <span className="ponto" aria-hidden="true" />
        {situacao}
      </p>
      <div className="redes" role="radiogroup" aria-label="Redes Wi-Fi">
        {redes.map((r) => (
          <button
            key={r.ssid}
            type="button"
            role="radio"
            aria-checked={escolhida?.ssid === r.ssid}
            aria-label={r.ssid}
            className="cartao-escolha"
            onClick={() => {
              setEscolhida(r);
              setErro("");
            }}
          >
            <b>{r.ssid}</b>
            <span>
              {r.emUso ? "conectada · " : ""}
              {r.protegida ? "com senha" : "aberta"} · sinal {r.sinal}%
            </span>
          </button>
        ))}
      </div>
      <Campo
        rotulo="Senha do Wi-Fi"
        type="password"
        value={senha}
        disabled={!escolhida?.protegida}
        onChange={(e) => setSenha(e.target.value)}
        mensagem={erro}
        erro={erro !== ""}
        autoComplete="off"
      />
      <p className="nota">A senha do Wi-Fi fica guardada só nesta máquina. Ela nunca vai para a imagem nem para o Tenancy.</p>
      <div className="acoes">
        {escolhida ? (
          <Botao
            variante="primario"
            disabled={ocupado || (escolhida.protegida && senha === "")}
            onClick={() => void conectar()}
          >
            Conectar
          </Botao>
        ) : (
          <Botao variante="primario" disabled={!rede?.conectada} onClick={() => aoConectar(nomeDaRede)}>
            Continuar
          </Botao>
        )}
      </div>
    </section>
  );
}
