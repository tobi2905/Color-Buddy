import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import MainWindow from "./MainWindow";

describe("MainWindow", () => {
  it("renders the app title", () => {
    render(<MainWindow />);
    expect(screen.getByRole("heading", { name: "Color-Buddy" })).toBeInTheDocument();
  });
});
