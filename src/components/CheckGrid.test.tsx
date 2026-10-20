import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { checks, palette } from "../test/fixtures";
import { CheckGrid } from "./CheckGrid";

const colors = palette.colors.map((c) => ({
  hex: c.hex,
  role: c.role,
  name: c.info.descriptive.name,
}));

describe("CheckGrid", () => {
  it("marks typical vision as the user's column too", () => {
    render(<CheckGrid colors={colors} checks={checks} userVision="normal" />);
    expect(screen.getByRole("columnheader", { name: "Typical (you)" })).toBeInTheDocument();
  });

  it("labels every verdict with an icon and words, not colour alone", () => {
    render(<CheckGrid colors={colors} checks={checks} userVision="deutan" />);
    const table = screen.getByRole("table");
    expect(within(table).getAllByText("Hard to tell").length).toBeGreaterThan(0);
    expect(within(table).getAllByText("Caution").length).toBeGreaterThan(0);
    expect(within(table).getAllByText("Distinct").length).toBeGreaterThan(0);
    expect(within(table).getAllByText("✕").length).toBeGreaterThan(0);
  });

  it("marks the user's column and the subtle pair", () => {
    render(<CheckGrid colors={colors} checks={checks} userVision="deutan" />);
    const header = screen.getByRole("columnheader", { name: "Deutan (you)" });
    expect(header).toBeInTheDocument();
    expect(screen.getByRole("rowheader", { name: /Background × Surface/ })).toHaveTextContent(
      "Subtle by design",
    );
  });

  it("summarises per vision type without counting subtle pairs", () => {
    render(<CheckGrid colors={colors} checks={checks} userVision="protan" />);
    const summary = screen.getByRole("list", { name: "Summary by colour vision type" });
    // Typical vision: the only non-subtle pairs are distinct.
    expect(within(summary).getByText("Typical").closest("li")).toHaveTextContent("all pairs");
    // Deutan: primary × accent at ΔE 6 is hard to tell.
    expect(within(summary).getByText("Deutan").closest("li")).toHaveTextContent("1 hard to tell");
    expect(within(summary).getByText("Protan").closest("li")).toHaveTextContent("1 to check");
  });

  it("says which subtle pair isn't counted, and only when there is one", () => {
    const { unmount } = render(<CheckGrid colors={colors} checks={checks} userVision="deutan" />);
    expect(screen.getByText(/Not counted above: Background × surface/)).toBeInTheDocument();
    unmount();
    render(
      <CheckGrid
        colors={colors}
        checks={checks.map((c) => ({ ...c, subtle: false }))}
        userVision="deutan"
      />,
    );
    expect(screen.queryByText(/Not counted above/)).toBeNull();
  });

  it("shows a WCAG level for text pairs", () => {
    render(<CheckGrid colors={colors} checks={checks} userVision="deutan" />);
    const row = screen.getByRole("rowheader", { name: /Background × Text/ }).closest("tr")!;
    expect(row).toHaveTextContent("15.80:1");
    expect(row).toHaveTextContent("AAA");
  });
});
