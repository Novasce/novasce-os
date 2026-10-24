import { expect, test, type Locator } from "@playwright/test";

const caixa = async (l: Locator) => {
  const b = await l.boundingBox();
  if (!b) throw new Error("elemento sem caixa");
  return b;
};

test("Pedir ajuda: o erro aparece no espaço reservado", async ({ page }) => {
  await page.goto("http://localhost:4311/?janela=pedir");
  await expect(page.getByText(/Dados da máquina/)).toBeVisible();
  await page.getByLabel("O que aconteceu?").fill("Travou.");
  const acoes = page.locator(".acoes-pedir");
  const antes = await caixa(acoes);
  await page.getByRole("button", { name: "Enviar para a Novasce" }).click();
  await expect(page.getByText(/O chamado não foi aberto/)).toBeVisible();
  expect(await caixa(acoes)).toEqual(antes);
});

test("Central: abrir um guia não mexe na coluna da esquerda nem rola a página", async ({ page }) => {
  await page.goto("http://localhost:4311/?janela=central&tela=comecar");
  const col = page.locator(".col-1");
  const antes = await caixa(col);
  const rolagem = await page.evaluate(() => window.scrollY);
  await page.getByRole("button", { name: "Primeiro dia" }).click();
  await expect(page.getByRole("button", { name: "Voltar" })).toBeVisible();
  expect(await caixa(col)).toEqual(antes);
  expect(await page.evaluate(() => window.scrollY)).toBe(rolagem);
});
