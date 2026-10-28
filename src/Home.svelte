<script lang="ts">
  import { onMount } from "svelte";
  import {
    Search,
    ArrowRight,
    ArrowUpRight,
    LockKeyhole,
    Sparkles,
    ShieldCheck,
  } from "lucide-svelte";
  import { host, type Engine } from "./core";
  let {
    bookmarks,
    engine,
    navigate,
    openBookmarks,
  }: {
    bookmarks: { url: string; title: string }[];
    engine: Engine;
    navigate: (text: string) => Promise<void>;
    openBookmarks: () => void;
  } = $props();
  let search = $state("");
  let clock = $state(new Date());
  onMount(() => {
    const timer = setInterval(() => (clock = new Date()), 60000);
    return () => clearInterval(timer);
  });
</script>

<div class="newtab-page">
  <div class="top-note">
    <span class="status-dot"></span> UN PEU D’ESPACE POUR VOS IDÉES
    <span class="edition">NOVA / 001</span>
  </div>
  <div class="home-center">
    <div class="greeting">
      <span class="short-line"></span> VOTRE PROCHAIN HORIZON
    </div>
    <h1>Le web, <em>en toute sérénité.</em></h1>
    <p class="intro">Moins de bruit. Plus de découvertes.</p>
    <form
      class="hero-search"
      onsubmit={(e) => {
        e.preventDefault();
        void navigate(search);
      }}
    >
      <Search size={21} /><input
        bind:value={search}
        aria-label="Rechercher sur le web"
        placeholder="Où voulez-vous aller ?"
      /><button title="Lancer la recherche"><ArrowRight size={19} /></button>
    </form>
    <div class="search-caption">
      Une recherche, une idée, une nouvelle perspective.<span
        >{engine === "duckduckgo"
          ? "DuckDuckGo"
          : engine === "brave"
            ? "Brave Search"
            : engine === "google"
              ? "Google"
              : "Bing"}
        <LockKeyhole size={10} /></span
      >
    </div>
    <div class="shortcuts-heading">
      <span>VOS POINTS DE DÉPART</span><button onclick={openBookmarks}
        >Tous les favoris <ArrowUpRight size={13} /></button
      >
    </div>
    <div class="shortcut-grid">
      {#each bookmarks.length ? bookmarks.slice(0, 4) : [{ title: "Wikipédia", url: "https://fr.wikipedia.org" }, { title: "GitHub", url: "https://github.com" }, { title: "DuckDuckGo", url: "https://duckduckgo.com" }, { title: "MDN Web Docs", url: "https://developer.mozilla.org/fr/" }] as site, i}<button
          class="shortcut"
          onclick={() => navigate(site.url)}
          ><div class={`shortcut-logo logo-${i}`}>
            {site.title === "GitHub"
              ? "⌘"
              : site.title === "Wikipédia"
                ? "W"
                : site.title === "DuckDuckGo"
                  ? "D"
                  : site.title[0]}<ArrowUpRight size={12} />
          </div>
          <strong>{site.title}</strong><span>{host(site.url)}</span></button
        >{/each}
    </div>
    <div class="thought">
      <Sparkles size={15} />
      <p>La curiosité est un très beau point de départ.</p>
      <span>Explorez à votre rythme.</span>
    </div>
  </div>
  <footer class="home-footer">
    <span
      ><ShieldCheck size={13} /> Un espace personnel, sans télémétrie ajoutée.</span
    ><span
      >{clock.toLocaleDateString("fr-FR", {
        weekday: "long",
        day: "numeric",
        month: "long",
      })}<span class="footer-dot">·</span>{clock.toLocaleTimeString("fr-FR", {
        hour: "2-digit",
        minute: "2-digit",
      })}</span
    >
  </footer>
</div>
