import type { ButtonHTMLAttributes } from "react";
import "./botao.css";

export type BotaoProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  variante?: "primario" | "secundario";
  tamanho?: "normal" | "pequeno";
  /** Quando existe, o botão fica desligado e o texto diz por quê (dica e leitor de tela). */
  desligado?: string;
};

export function Botao({ variante = "secundario", tamanho = "normal", desligado, className, type = "button", onClick, title, ...resto }: BotaoProps) {
  const classes = ["nv-botao", `nv-botao--${variante}`, tamanho === "pequeno" ? "nv-botao--pequeno" : "", desligado ? "nv-botao--desligado" : "", className]
    .filter(Boolean)
    .join(" ");
  return (
    <button
      type={type}
      className={classes}
      aria-disabled={desligado ? true : undefined}
      title={desligado ?? title}
      onClick={desligado ? undefined : onClick}
      {...resto}
    />
  );
}
