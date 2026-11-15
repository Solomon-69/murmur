// Murmur Cloud: a tiny relay so Murmur's prompt mode and clean-up work out of
// the box. The Groq key lives here as a secret (GROQ_API_KEY), never in the
// app. It only accepts Murmur's own short rewrite requests.

const GROQ_URL = "https://api.groq.com/openai/v1/chat/completions";
const MODEL = "openai/gpt-oss-20b";
const APP_TOKEN = "murmur-app-v1";
const MAX_BODY_BYTES = 24_000;
const MAX_OUTPUT_TOKENS = 1500;
// Per address, per minute, per server copy. Best effort; stops runaway loops.
const PER_MINUTE = 20;
const hits = new Map();

function tooMany(ip) {
  const minute = Math.floor(Date.now() / 60_000);
  const key = `${ip}:${minute}`;
  const n = (hits.get(key) || 0) + 1;
  hits.set(key, n);
  if (hits.size > 5000) hits.clear();
  return n > PER_MINUTE;
}

function reply(status, body) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json" },
  });
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    if (request.method === "GET" && url.pathname === "/") {
      return reply(200, { ok: true, service: "murmur-cloud" });
    }
    if (request.method !== "POST" || !url.pathname.endsWith("/chat/completions")) {
      return reply(404, { error: "not found" });
    }
    if (request.headers.get("authorization") !== `Bearer ${APP_TOKEN}`) {
      return reply(401, { error: "unknown app" });
    }
    if (!env.GROQ_API_KEY) return reply(503, { error: "relay not set up" });
    const ip = request.headers.get("cf-connecting-ip") || "unknown";
    if (tooMany(ip)) return reply(429, { error: "slow down" });

    const raw = await request.text();
    if (raw.length > MAX_BODY_BYTES) return reply(413, { error: "too long" });
    let input;
    try {
      input = JSON.parse(raw);
    } catch {
      return reply(400, { error: "bad request" });
    }
    const messages = Array.isArray(input.messages)
      ? input.messages
          .filter((m) => m && ["system", "user"].includes(m.role) && typeof m.content === "string")
          .slice(0, 4)
          .map((m) => ({ role: m.role, content: m.content }))
      : [];
    if (messages.length === 0) return reply(400, { error: "no text" });

    const upstream = await fetch(GROQ_URL, {
      method: "POST",
      headers: {
        authorization: `Bearer ${env.GROQ_API_KEY}`,
        "content-type": "application/json",
      },
      body: JSON.stringify({
        model: MODEL,
        messages,
        temperature: 0,
        reasoning_effort: "low",
        max_completion_tokens: MAX_OUTPUT_TOKENS,
      }),
    });
    // Pass Groq's answer through; never log the dictated text.
    return new Response(upstream.body, {
      status: upstream.status,
      headers: { "content-type": "application/json" },
    });
  },
};
