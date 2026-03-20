import { createSign } from "node:crypto";
import { httpRouter } from "convex/server";
import { httpAction } from "./_generated/server";
import { api } from "./_generated/api";

const http = httpRouter();

http.route({
  path: "/extension/signed-update",
  method: "GET",
  handler: httpAction(async (ctx) => {
    let payload = await ctx.runQuery(api.extension.getLatestPublicSnapshot, {});
    if (!payload) {
      await ctx.runMutation(api.extension.refreshGlobalSnapshot, {});
      payload = await ctx.runQuery(api.extension.getLatestPublicSnapshot, {});
    }

    const privateKey = process.env.EXTENSION_SIGNING_PRIVATE_KEY_PEM;
    const signature = privateKey
      ? createSign("RSA-SHA256")
          .update(JSON.stringify(payload))
          .end()
          .sign(privateKey, "base64")
      : "";

    return new Response(
      JSON.stringify({
        data: payload,
        signature,
      }),
      {
        status: 200,
        headers: {
          "content-type": "application/json",
          "cache-control": "public, max-age=300",
        },
      },
    );
  }),
});

export default http;
