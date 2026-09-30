export function getSupabaseConfig(): { url: string; key: string } | null {
  const url = process.env.NEXT_PUBLIC_SUPABASE_URL;
  const key = process.env.NEXT_PUBLIC_SUPABASE_PUBLISHABLE_KEY;
  if (!url || !key || url.includes("your-project") || key === "your-publishable-key") return null;
  try {
    const parsed = new URL(url);
    const local = ["localhost", "127.0.0.1"].includes(parsed.hostname);
    if (parsed.protocol !== "https:" && !(local && parsed.protocol === "http:")) return null;
    if (parsed.username || parsed.password || parsed.search || parsed.hash) return null;
    if (key.startsWith("sb_secret_")) return null;
    // Legacy service-role JWTs must never be accepted as public configuration.
    if (key.split(".").length === 3) {
      const claims = JSON.parse(Buffer.from(key.split(".")[1], "base64url").toString()) as { role?: string };
      if (claims.role !== "anon") return null;
    }
    return { url, key };
  } catch {
    return null;
  }
}
