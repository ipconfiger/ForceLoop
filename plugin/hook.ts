import type { Plugin } from "@opencode-ai/plugin";

export default (async (ctx) => {
  const { client, $ } = ctx;

  return {
    event: async ({ event }) => {
      if (event.type !== "session.idle") return;

      const sessionID = event.properties.sessionID;

      // Call fl gate; on failure, inject output back to AI as prompt.
      const result = await $`fl gate`.timeout(60_000);

      if (result.exitCode !== 0) {
        // Non-zero exit → re-inject stdout+stderr as prompt so the
        // AI can see the gate reason and auto-fix.
        client.session.prompt({
          path: { id: sessionID },
          body: {
            noReply: false,
            parts: [{
              type: "text",
              text: result.stdout + result.stderr,
            }],
          },
        });
      }
      // Exit 0: silent pass, no AI intervention.
    },
  };
}) satisfies Plugin;
