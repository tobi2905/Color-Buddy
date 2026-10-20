// @vitest-environment node
/// <reference types="node" />
/**
 * Phase 7 acceptance: the committed sample exports (written by the Rust export tests) are valid:
 * the CSS parses, the JSON has the documented shape, and the Tailwind snippet loads as a config.
 */
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import postcss from "postcss";
import { compile } from "tailwindcss";
import { describe, expect, it } from "vitest";

const dir = fileURLToPath(new URL("../../src-tauri/tests/fixtures/exports", import.meta.url));
const read = (file: string) => readFileSync(resolve(dir, file), "utf8");
const KEYS = ["background", "surface", "primary", "accent", "accent-2", "text"];

describe("sample exports", () => {
  it("CSS parses into one :root rule with a variable per colour", () => {
    const root = postcss.parse(read("sky-afternoon.css"));
    const rules = root.nodes.filter((n) => n.type === "rule");
    expect(rules).toHaveLength(1);
    const rule = rules[0] as postcss.Rule;
    expect(rule.selector).toBe(":root");
    const decls = rule.nodes.filter((n): n is postcss.Declaration => n.type === "decl");
    expect(decls.map((d) => d.prop)).toEqual(KEYS.map((k) => `--color-${k}`));
    for (const d of decls) expect(d.value).toMatch(/^#[0-9A-F]{6}$/);
  });

  it("JSON validates against the documented shape", () => {
    const doc = JSON.parse(read("sky-afternoon.json"));
    expect(doc).toMatchObject({ name: "Sky afternoon", scheme: "splitComplementary" });
    expect(doc.colors.map((c: { key: string }) => c.key)).toEqual(KEYS);
    for (const c of doc.colors) {
      expect(c.hex).toMatch(/^#[0-9A-F]{6}$/);
      expect(c.rgb).toHaveLength(3);
      expect(Object.keys(c.names)).toEqual(["basic", "descriptive", "precise"]);
    }
    for (const v of ["normal", "protan", "deutan", "tritan", "achromat"]) {
      expect(doc.checks.byVision[v]).toEqual({
        hardToTell: expect.any(Number),
        caution: expect.any(Number),
      });
    }
    for (const p of doc.checks.problems) {
      expect(KEYS).toEqual(expect.arrayContaining(p.pair));
      expect(["caution", "hardToTell"]).toContain(p.verdict);
    }
  });

  it("Tailwind snippet loads as a config and produces colour utilities", async () => {
    const source = read("sky-afternoon.tailwind.js");
    const url = `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;
    const config = (await import(/* @vite-ignore */ url)).default;
    expect(Object.keys(config.theme.extend.colors["sky-afternoon"])).toEqual(KEYS);

    const compiler = await compile('@config "./tailwind.config.js";\n@tailwind utilities;', {
      base: dir,
      loadModule: async () => ({ path: "tailwind.config.js", base: dir, module: config }),
      loadStylesheet: async () => ({ path: "", base: dir, content: "" }),
    });
    const css = compiler.build(["bg-sky-afternoon-primary", "text-sky-afternoon-accent-2"]);
    expect(css).toMatch(/\.bg-sky-afternoon-primary\s*\{[^}]*#87CEEB/i);
    expect(css).toMatch(/\.text-sky-afternoon-accent-2\s*\{[^}]*#65440A/i);
  });

  it("text formats match their descriptions", () => {
    const hex = read("sky-afternoon.hex.txt").trimEnd().split("\n");
    expect(hex).toHaveLength(6);
    for (const h of hex) expect(h).toMatch(/^#[0-9A-F]{6}$/);
    const names = read("sky-afternoon.names.txt").trimEnd().split("\n");
    expect(names).toEqual([
      "alice blue",
      "alice blue",
      "sky blue",
      "indian red",
      "saddle brown",
      "midnight blue",
    ]);
    expect(read("sky-afternoon.names-hex.txt").trimEnd().split("\n")).toEqual(
      names.map((n: string, i: number) => `${n} ${hex[i]}`),
    );
    expect(read("sky-afternoon.detailed.txt").split("\n")[2]).toBe(
      "Primary: sky blue, #87CEEB, rgb(135, 206, 235)",
    );
  });
});
