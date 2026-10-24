import { useId, type InputHTMLAttributes } from "react";
import "./campo.css";

export type CampoProps = InputHTMLAttributes<HTMLInputElement> & {
  rotulo: string;
  mensagem?: string;
  erro?: boolean;
};

export function Campo({ rotulo, mensagem, erro = false, id, className, ...resto }: CampoProps) {
  const gerado = useId();
  const idDoCampo = id ?? gerado;
  const idDaMensagem = `${idDoCampo}-mensagem`;
  return (
    <div className={["nv-campo", className].filter(Boolean).join(" ")}>
      <label className="nv-campo__rotulo" htmlFor={idDoCampo}>
        {rotulo}
      </label>
      <input
        id={idDoCampo}
        className="nv-campo__entrada"
        aria-invalid={erro || undefined}
        aria-describedby={idDaMensagem}
        {...resto}
      />
      <p
        id={idDaMensagem}
        className={erro ? "nv-campo__mensagem nv-campo__mensagem--erro" : "nv-campo__mensagem"}
        aria-live="polite"
      >
        {mensagem ?? ""}
      </p>
    </div>
  );
}
