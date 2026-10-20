import { describe, expect, it } from "vitest";
import { parseColorInput } from "./parseColorInput";

describe("parseColorInput", () => {
  it.each([
    ["#87ceeb", "#87CEEB"],
    ["87CEEB", "#87CEEB"],
    [" #abc ", "#AABBCC"],
    ["#000", "#000000"],
    ["a1b", "#AA11BB"],
    ["facade", "#FACADE"],
  ])("reads hex %s", (input, hex) => {
    expect(parseColorInput(input)).toEqual({ kind: "color", hex, source: "hex" });
  });

  it.each([
    ["rgb(135, 206, 235)", "#87CEEB"],
    ["rgb(135 206 235 / 50%)", "#87CEEB"],
    ["RGBA(0,0,0,0.5)", "#000000"],
    ["135, 206, 235", "#87CEEB"],
    ["255 0 10", "#FF000A"],
  ])("reads rgb %s", (input, hex) => {
    expect(parseColorInput(input)).toEqual({ kind: "color", hex, source: "rgb" });
  });

  it("treats words as names, even hex-looking ones", () => {
    expect(parseColorInput("sky blue")).toEqual({ kind: "name", query: "sky blue" });
    expect(parseColorInput("bed")).toEqual({ kind: "name", query: "bed" });
    expect(parseColorInput("Sky-Blue")).toEqual({ kind: "name", query: "Sky-Blue" });
  });

  it("explains invalid input", () => {
    expect(parseColorInput("rgb(300, 0, 0)")).toEqual({
      kind: "invalid",
      message: "RGB values go from 0 to 255.",
    });
    expect(parseColorInput("#12345")).toMatchObject({ kind: "invalid" });
    expect(parseColorInput("   ")).toEqual({ kind: "empty" });
  });
});
