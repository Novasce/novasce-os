import type { ButtonHTMLAttributes } from "react";
import "./botao.css";

export type BotaoProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  variante?: "primario" | "secundario";
};

export function Botao({ variante = "secundario", className, type = "button", ...resto }: BotaoProps) {
  const classes = ["nv-botao", `nv-botao--${variante}`, className].filter(Boolean).join(" ");
  return <button type={type} className={classes} {...resto} />;
}
