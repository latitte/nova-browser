export type Engine = "duckduckgo" | "google" | "bing" | "brave";
export const engines: Record<Engine, string> = {
  duckduckgo: "https://duckduckgo.com/?q=",
  google: "https://www.google.com/search?q=",
  bing: "https://www.bing.com/search?q=",
  brave: "https://search.brave.com/search?q=",
};
export function address(input: string, engine: Engine = "duckduckgo"): string {
  const text = input.trim();
  if (!text) throw new Error("Saisissez une adresse ou une recherche.");
  if (
    /^[a-z][a-z0-9+.-]*:/i.test(text) &&
    !/^localhost:\d+/i.test(text) &&
    !/^[\w.-]+:\d+(\/|$)/.test(text)
  ) {
    const url = new URL(text);
    if (
      !["http:", "https:"].includes(url.protocol) ||
      !url.hostname ||
      url.username ||
      url.password
    )
      throw new Error(
        "Cette adresse n’est pas autorisée. Utilisez HTTP ou HTTPS.",
      );
    return url.href;
  }
  if (
    !/\s/.test(text) &&
    (/^[\w-]+(\.[\w-]+)+(:\d+)?(\/|\?|#|$)/.test(text) ||
      /^localhost(:\d+)?(\/|$)/.test(text) ||
      /^127\.0\.0\.1/.test(text))
  ) {
    return new URL(
      (/^(localhost|127\.0\.0\.1)/.test(text) ? "http://" : "https://") + text,
    ).href;
  }
  return engines[engine] + encodeURIComponent(text);
}
export function host(url: string): string {
  try {
    return new URL(url).hostname.replace(/^www\./, "");
  } catch {
    return "Nouvel onglet";
  }
}
