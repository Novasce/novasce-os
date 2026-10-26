import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

// Sem `globals: true` o Testing Library não limpa sozinho: cada teste começa com a tela vazia.
afterEach(() => cleanup());
