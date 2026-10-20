import { invoke } from "@tauri-apps/api/core";
import { vi } from "vitest";

type Handler = (args: Record<string, unknown>) => unknown;

/**
 * Route mocked `invoke` calls by command name. Requires in the test file:
 *   vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
 *   vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(() => Promise.resolve(() => {})) }));
 */
export function mockCommands(handlers: Record<string, Handler>) {
  vi.mocked(invoke).mockImplementation(async (cmd: string, args?: unknown) => {
    const handler = handlers[cmd];
    if (!handler) return undefined;
    return handler((args ?? {}) as Record<string, unknown>);
  });
}
