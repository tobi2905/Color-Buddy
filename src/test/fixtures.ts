import type { ColorDetails, ColorInfo, PairCheck, Palette } from "../types/color";

export const skyBlue: ColorInfo = {
  hex: "#87CEEB",
  rgb: { r: 135, g: 206, b: 235 },
  oklch: [0.81, 0.09, 222.6],
  basic: { name: "blue", hex: "#75BBFD", deltaE: 6.1 },
  basicAlternative: null,
  descriptive: { name: "sky blue", hex: "#87CEEB", deltaE: 0 },
  precise: { name: "light blue", hex: "#95D0FC", deltaE: 4.9 },
  description: "light, slightly desaturated, cool blue",
  values: {
    hex: "#87CEEB",
    rgb: "rgb(135, 206, 235)",
    hsl: "hsl(197, 71%, 73%)",
    oklch: "oklch(81.4% 0.088 225.3)",
  },
};

export const info = (hex: string, name: string, basic = "blue"): ColorInfo => ({
  ...skyBlue,
  hex,
  basic: { name: basic, hex, deltaE: 3 },
  descriptive: { name, hex, deltaE: 0 },
  precise: { name: `precise ${name}`, hex, deltaE: 2 },
  values: { hex, rgb: `rgb(of ${hex})`, hsl: `hsl(of ${hex})`, oklch: `oklch(of ${hex})` },
});

export const skyBlueDetails: ColorDetails = {
  info: skyBlue,
  references: ["clear daytime sky", "bluebirds", "summer days"],
  associations: ["calm", "openness", "freedom", "airlines", "large areas", "summer holidays"],
  pairings: ["#FFFFFF", "#FFFF00"],
  referenceSource: {
    name: "sky blue",
    hex: "#87CEEB",
    match: "exact",
    deltaE: 0,
    label: "sky blue",
  },
  confusedWith: [{ name: "light grey", hex: "#D3D3D3", deltaE: 3.2 }],
};

export const palette: Palette = {
  scheme: "complementary",
  baseHex: "#87CEEB",
  explanation: "sky blue with its opposite, peach puff: the strongest hue contrast.",
  colors: [
    { hex: "#F3F9FF", role: "background", info: info("#F3F9FF", "alice blue") },
    { hex: "#DEEDFB", role: "surface", info: info("#DEEDFB", "alice blue") },
    { hex: "#87CEEB", role: "primary", info: skyBlue },
    { hex: "#E8A27C", role: "accent", info: info("#E8A27C", "dark salmon", "orange") },
    { hex: "#132331", role: "text", info: info("#132331", "midnight blue") },
  ],
};

const result = (de: number[]): PairCheck["results"] => {
  const verdict = (d: number, i: number) =>
    d >= (i === 0 ? 15 : 20) ? "distinct" : d >= (i === 0 ? 8 : 10) ? "caution" : "hardToTell";
  return (["normal", "protan", "deutan", "tritan", "achromat"] as const).map((v, i) => [
    v,
    de[i],
    verdict(de[i], i),
  ]);
};

export const checks: PairCheck[] = [
  { a: 0, b: 1, results: result([4, 4, 4, 4, 4]), contrastRatio: 1.1, subtle: true },
  { a: 2, b: 3, results: result([40, 12, 6, 30, 3]), contrastRatio: 1.2, subtle: false },
  { a: 0, b: 4, results: result([90, 90, 90, 90, 90]), contrastRatio: 15.8, subtle: false },
];
