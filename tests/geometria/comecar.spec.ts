import { expect, test, type Locator } from "@playwright/test";

const caixa = async (l: Locator) => {
  const b = await l.boundingBox();
  if (!b) throw new Error("elemento sem caixa");
  return b;
};

test("passo 1: carregando e com código ocupam o mesmo lugar", async ({ page }) => {
  await page.goto("http://localhost:4310/?tela=passo1");
  await expect(page.getByText("Passo 1 · Tenancy")).toBeVisible();
  const cartao = page.locator(".cartao.sel").first();
  const ficha = page.locator(".ficha").first();
  const antes = [await caixa(cartao), await caixa(ficha)];
  await expect(page.getByText("ABCD-EFGH")).toBeVisible();
  expect([await caixa(cartao), await caixa(ficha)]).toEqual(antes);
});

test("hover, foco e pressionado só mudam a cor do botão", async ({ page }) => {
  await page.goto("http://localhost:4310/?tela=passo1");
  await expect(page.getByText("ABCD-EFGH")).toBeVisible();
  const b = page.getByRole("button", { name: "Copiar" });
  const antes = await caixa(b);
  await b.hover();
  expect(await caixa(b)).toEqual(antes);
  await b.focus();
  expect(await caixa(b)).toEqual(antes);
  await page.mouse.down();
  expect(await caixa(b)).toEqual(antes);
  await page.mouse.up();
});

test("mostrar o comando por trás troca o conteúdo no mesmo lugar", async ({ page }) => {
  await page.goto("http://localhost:4310/?tela=passo1");
  const ficha = page.locator(".ficha").first();
  const antes = await caixa(ficha);
  await page.getByRole("button", { name: "Mostrar o comando por trás" }).click();
  expect(await caixa(ficha)).toEqual(antes);
});
