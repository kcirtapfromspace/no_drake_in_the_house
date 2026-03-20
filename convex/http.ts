import { httpRouter } from "convex/server";
import { httpAction } from "./_generated/server";
import { api, internal } from "./_generated/api";

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

    const payloadJson = JSON.stringify(payload);
    const signature: string = await ctx.runAction(
      internal.signing.signPayload,
      { payload: payloadJson },
    );

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
