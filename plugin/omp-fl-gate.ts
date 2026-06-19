import type { ExtensionAPI } from "@oh-my-pi/pi-coding-agent";
import { $ } from "bun";

// ForceLoop `fl gate` hook for oh-my-pi (omp).
//
// Behavior:
// - On `session_stop`, run `fl gate` via Bun Shell.
// - Exit 0: silent pass (return undefined).
// - Exit != 0: return `{ continue: true, additionalContext }` to inject
//   gate output as a new user message, driving the agent's auto-fix loop.
//
// omp limits continuation to 8 consecutive turns per session_stop.

export default function flGateHook(pi: ExtensionAPI): void {
  pi.setLabel("ForceLoop Gate");

  pi.on("session_stop", async (_event, _ctx) => {
    let text = "";
    try {
      const result = await $`fl gate`.nothrow();
      if (result.exitCode === 0) return; // gate passed, silent
      text = (result.stdout?.toString() ?? "") + (result.stderr?.toString() ?? "");
    } catch (err) {
      text = String(err);
    }

    if (!text.trim()) return;

    // Inject gate output → agent auto-fixes in next turn
    return {
      continue: true,
      additionalContext: text,
    };
  });
}