import type { Plugin } from "@opencode-ai/plugin"

// ForceLoop `fl gate` auto-driver for OpenCode.
//
// Behavior:
// - On `session.idle`, run `fl gate` with a 60-second timeout.
// - Exit 0: silent pass, no AI intervention.
// - Exit != 0: re-inject stdout+stderr as a prompt into the session
//   with `noReply: false` to trigger the AI's auto-reply / fix loop.
//
// Source of truth for OpenCode plugin API:
//   https://opencode.ai/docs/plugins/

export const FlGateHook: Plugin = async ({ client, $ }) => {
  return {
    event: async ({ event }) => {
      if (event.type !== "session.idle") return;

      const sessionID = event.properties?.sessionID;
      if (!sessionID) return;

      const result = await $`fl gate`.nothrow();

      if (result.exitCode !== 0) {
        const text = (result.stdout?.toString() ?? "") + (result.stderr?.toString() ?? "");
        await client.session.prompt({
          path: { id: sessionID },
          body: {
            noReply: false,
            parts: [{ type: "text", text }],
          },
        });
      }
    },
  };
};