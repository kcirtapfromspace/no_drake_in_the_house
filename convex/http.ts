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

http.route({
  path: "/api/v1/apple-music/auth/developer-token",
  method: "GET",
  handler: httpAction(async (ctx) => {
    const result = await ctx.runAction(
      internal.signing.getDeveloperToken,
      {},
    );

    return new Response(JSON.stringify({ success: true, data: result }), {
      status: 200,
      headers: {
        "content-type": "application/json",
        "cache-control": "public, max-age=1800",
      },
    });
  }),
});

http.route({
  path: "/stripe/webhook",
  method: "POST",
  handler: httpAction(async (ctx, req) => {
    const signature = req.headers.get("stripe-signature");
    if (!signature) {
      return new Response(
        JSON.stringify({ error: "Missing stripe-signature header" }),
        {
          status: 400,
          headers: { "content-type": "application/json" },
        },
      );
    }

    const payload = await req.text();

    try {
      await ctx.runAction((internal as any).stripeActions.handleWebhookEvent, {
        payload,
        signature,
      });

      return new Response(JSON.stringify({ received: true }), {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    } catch (error: any) {
      console.error("Stripe webhook error:", error);
      return new Response(
        JSON.stringify({ error: "Webhook processing failed" }),
        {
          status: 400,
          headers: { "content-type": "application/json" },
        },
      );
    }
  }),
});

export default http;
