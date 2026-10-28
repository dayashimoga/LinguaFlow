export interface Env {
  ENVIRONMENT: string;
  API_VERSION: string;
  GATEWAY_URL?: string;
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);

    // Standard CORS headers
    const corsHeaders = {
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
      "Access-Control-Allow-Headers": "Content-Type, Authorization",
    };

    if (request.method === "OPTIONS") {
      return new Response(null, { headers: corsHeaders });
    }

    // Edge health check
    if (url.pathname === "/edge/health") {
      return new Response(
        JSON.stringify({
          status: "healthy",
          edge_region: request.cf?.colo || "local",
          environment: env.ENVIRONMENT,
          version: env.API_VERSION,
          timestamp: new Date().toISOString(),
        }),
        {
          headers: {
            ...corsHeaders,
            "Content-Type": "application/json",
          },
        }
      );
    }

    // Edge capability lookup
    if (url.pathname === "/edge/capabilities") {
      return new Response(
        JSON.stringify({
          supported_languages: ["kn-IN", "ta-IN", "en-US"],
          stage_0_baseline: true,
          offline_first_preferred: true,
        }),
        {
          headers: {
            ...corsHeaders,
            "Content-Type": "application/json",
          },
        }
      );
    }

    // Proxy to primary gateway if configured
    if (env.GATEWAY_URL) {
      const targetUrl = new URL(url.pathname + url.search, env.GATEWAY_URL);
      return fetch(new Request(targetUrl, request));
    }

    return new Response(
      JSON.stringify({
        message: "LinguaFlow Edge Service active. Stage 0 Foundation baseline.",
      }),
      {
        headers: {
          ...corsHeaders,
          "Content-Type": "application/json",
        },
      }
    );
  },
};
