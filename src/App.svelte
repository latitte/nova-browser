<script lang="ts">
  import { onMount, tick } from "svelte";
  import Home from "./Home.svelte";
  import Settings from "./Settings.svelte";
  import { invoke, isTauri } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import {
    ArrowLeft,
    ArrowRight,
    RotateCw,
    X,
    Plus,
    Search,
    Star,
    Settings as SettingsIcon,
    History,
    PanelLeftClose,
    PanelLeftOpen,
    Globe2,
    ShieldCheck,
    LockKeyhole,
    ArrowUpRight,
    Trash2,
    MoreHorizontal,
    Copy,
    Pin,
  } from "lucide-svelte";
  import { address, host, type Engine } from "./core";
  type Tab = {
    id: string;
    url: string;
    title: string;
    pinned: boolean;
    loading: boolean;
  };
  type Entry = { url: string; title: string };
  const native = isTauri();
  let tabs = $state<Tab[]>([]),
    active = $state(""),
    closed: Tab[] = [];
  let bookmarks = $state<Entry[]>([]),
    history = $state<Entry[]>([]);
  let panel = $state<"settings" | "history" | "bookmarks" | null>(null);
  let prefersDark = $state(true);
  let theme = $state("dark"),
    accent = $state("#b8c5a4"),
    vertical = $state(true),
    engine = $state<Engine>("duckduckgo");
  let query = $state(""),
    filter = $state(""),
    error = $state(""),
    menu = $state(false),
    zoom = $state(1);
  let ready = $state(false),
    addressing = $state(false),
    busy = false,
    resync = false,
    dragged = "";
  let viewport: HTMLDivElement, addressInput: HTMLInputElement;
  const created = new Set<string>();
  let current = $derived(tabs.find((t) => t.id === active));
  let suggestions = $derived(
    query.length > 1
      ? [...bookmarks, ...history]
          .filter(
            (e, i, a) =>
              a.findIndex((x) => x.url === e.url) === i &&
              (e.url + e.title).toLowerCase().includes(query.toLowerCase()),
          )
          .slice(0, 5)
      : [],
  );
  let isFavorite = $derived(bookmarks.some((b) => b.url === current?.url));
  let dark = $derived(theme === "dark" || (theme === "auto" && prefersDark));
  function report(e: unknown) {
    error = String(e);
    void syncView();
  }
  async function call<T>(
    command: string,
    args: Record<string, unknown> = {},
  ): Promise<T> {
    if (!native)
      throw new Error(
        "La navigation nécessite l’application desktop NOVA. Lancez npm run tauri dev.",
      );
    return invoke<T>(command, args);
  }
  function fresh(url = ""): Tab {
    return {
      id: `tab-${crypto.randomUUID()}`,
      url,
      title: url ? host(url) : "Nouvel onglet",
      pinned: false,
      loading: false,
    };
  }
  async function persist() {
    if (!ready) return;
    const value = JSON.stringify({
      theme,
      accent,
      vertical,
      engine,
      tabs: tabs.map(({ url, pinned }) => ({ url, pinned })),
      active: tabs.findIndex((t) => t.id === active),
    });
    try {
      if (native) await call("write_data", { kind: "settings", value });
      else localStorage.setItem("nova-settings", value);
    } catch (e) {
      report(e);
    }
  }
  async function action(id: string, action: string, value?: string) {
    if (created.has(id)) await call("tab_action", { id, action, value });
  }
  function bounds() {
    const b = viewport.getBoundingClientRect();
    return { x: b.x, y: b.y, width: b.width, height: b.height };
  }
  async function syncView() {
    if (!ready || !native || !viewport) return;
    if (busy) {
      resync = true;
      return;
    }
    busy = true;
    const id = active;
    try {
      for (const key of created) await action(key, "hide");
      await tick();
      const tab = tabs.find((t) => t.id === id);
      if (tab?.url && !panel && !menu && !addressing && !error) {
        if (!created.has(id)) {
          await call("open_tab", { id, url: tab.url, ...bounds() });
          created.add(id);
          if (!tabs.some((t) => t.id === id)) {
            await action(id, "close");
            created.delete(id);
            return;
          }
          if (id !== active || panel || menu || addressing || error) {
            await action(id, "hide");
            resync = true;
            return;
          }
        }
        await call("layout", { id, ...bounds() });
        await action(id, "show");
      }
    } catch (e) {
      report(e);
    } finally {
      busy = false;
      if (resync || id !== active) {
        resync = false;
        void syncView();
      }
    }
  }
  async function select(id: string) {
    active = id;
    query = tabs.find((t) => t.id === id)?.url || "";
    panel = null;
    menu = false;
    error = "";
    await syncView();
    await persist();
  }
  async function add(url = "") {
    const tab = fresh(url);
    tabs.push(tab);
    await select(tab.id);
  }
  async function close(id: string) {
    const i = tabs.findIndex((t) => t.id === id);
    if (i < 0) return;
    if (tabs[i].pinned) {
      report("Désépinglez cet onglet avant de le fermer.");
      return;
    }
    try {
      await action(id, "close");
      created.delete(id);
      closed.push(tabs[i]);
      if (closed.length > 20) closed.shift();
      tabs.splice(i, 1);
      if (!tabs.length) await add();
      else if (active === id)
        await select(tabs[Math.min(i, tabs.length - 1)].id);
      await persist();
    } catch (e) {
      report(e);
    }
  }
  async function restore() {
    const tab = closed.pop();
    if (tab) await add(tab.url);
  }
  async function navigate(text: string) {
    try {
      const url = address(text, engine);
      error = "";
      query = url;
      panel = null;
      menu = false;
      addressing = false;
      addressInput?.blur();
      if (!native)
        throw new Error(
          "Ouvrez NOVA en application desktop pour naviguer sur Internet. Le mode web présente uniquement l’interface locale.",
        );
      if (!current) await add();
      const tab = tabs.find((t) => t.id === active)!;
      tab.url = url;
      tab.title = host(url);
      tab.loading = true;
      if (created.has(active)) await action(active, "navigate", url);
      await syncView();
      await persist();
    } catch (e) {
      report(e);
    }
  }
  async function togglePanel(next: typeof panel) {
    panel = panel === next ? null : next;
    menu = false;
    filter = "";
    if (panel === "history")
      try {
        history = await call<Entry[]>("read_data", { kind: "history" });
      } catch (e) {
        report(e);
      }
    await syncView();
  }
  async function bookmark() {
    if (!current?.url) return;
    try {
      await call("write_data", {
        kind: isFavorite ? "unbookmark" : "bookmark",
        value: isFavorite
          ? current.url
          : JSON.stringify({ url: current.url, title: current.title }),
      });
      bookmarks = await call<Entry[]>("read_data", { kind: "bookmarks" });
    } catch (e) {
      report(e);
    }
  }
  async function removeBookmark(url: string) {
    try {
      await call("write_data", { kind: "unbookmark", value: url });
      bookmarks = bookmarks.filter((b) => b.url !== url);
    } catch (e) {
      report(e);
    }
  }
  async function clearHistory() {
    try {
      await call("write_data", { kind: "clear_history", value: "" });
      history = [];
    } catch (e) {
      report(e);
    }
  }
  async function clearSites() {
    try {
      await call("clear_site_data");
      error = "Les cookies et données des sites ouverts ont été supprimés.";
    } catch (e) {
      report(e);
    }
  }
  async function settingsChanged() {
    await tick();
    await persist();
    await syncView();
  }
  async function doAction(a: string) {
    try {
      await action(active, a);
    } catch (e) {
      report(e);
    }
  }
  function drop(target: string) {
    const from = tabs.findIndex((t) => t.id === dragged),
      to = tabs.findIndex((t) => t.id === target);
    if (from >= 0 && to >= 0) {
      const [tab] = tabs.splice(from, 1);
      tabs.splice(to, 0, tab);
      void persist();
    }
  }
  async function keydown(e: KeyboardEvent) {
    if (!(e.ctrlKey || e.metaKey)) return;
    const k = e.key.toLowerCase();
    if (
      ["l", "t", "w", ",", "h", "d"].includes(k) ||
      (k === "t" && e.shiftKey)
    ) {
      e.preventDefault();
      if (k === "l") {
        addressInput?.focus();
        addressInput?.select();
      }
      if (k === "t") e.shiftKey ? await restore() : await add();
      if (k === "w") await close(active);
      if (k === ",") await togglePanel("settings");
      if (k === "h") await togglePanel("history");
      if (k === "d") await bookmark();
    }
  }
  onMount(() => {
    const media = matchMedia("(prefers-color-scheme: dark)");
    prefersDark = media.matches;
    const mediaChange = (e: MediaQueryListEvent) => {
      prefersDark = e.matches;
    };
    media.addEventListener("change", mediaChange);
    let alive = true;
    const cleanup: (() => void)[] = [];
    async function init() {
      try {
        const saved: Record<string, unknown> = native
          ? await call<Record<string, unknown>>("read_data", {
              kind: "settings",
            })
          : JSON.parse(localStorage.getItem("nova-settings") || "{}");
        if (["dark", "light", "auto"].includes(String(saved.theme)))
          theme = String(saved.theme);
        if (
          typeof saved.accent === "string" &&
          /^#[0-9a-f]{6}$/i.test(saved.accent)
        )
          accent = saved.accent;
        if (typeof saved.vertical === "boolean") vertical = saved.vertical;
        if (
          ["google", "bing", "duckduckgo", "brave"].includes(
            String(saved.engine),
          )
        )
          engine = saved.engine as Engine;
        if (Array.isArray(saved.tabs))
          tabs = saved.tabs
            .slice(0, 50)
            .filter((t) => typeof t.url === "string")
            .map((t) => ({ ...fresh(t.url), pinned: !!t.pinned }));
        if (!tabs.length) tabs = [fresh()];
        active = tabs[Number(saved.active)]?.id || tabs[0].id;
        query = current?.url || "";
        if (native) {
          bookmarks = await call<Entry[]>("read_data", { kind: "bookmarks" });
          history = await call<Entry[]>("read_data", { kind: "history" });
          const unpage = await listen<{
            id: string;
            url: string;
            title: string | null;
            loading: boolean;
          }>("page", ({ payload: p }) => {
            const t = tabs.find((t) => t.id === p.id);
            if (t) {
              t.url = p.url;
              t.loading = p.loading;
              if (p.title) t.title = p.title;
              if (active === p.id) query = p.url;
              if (!p.loading) void persist();
            }
          });
          cleanup.push(unpage);
          const unnotice = await listen<string>("notice", ({ payload }) =>
            report(payload),
          );
          cleanup.push(unnotice);
          const unpopup = await listen<string>("popup", ({ payload }) => {
            void add(payload);
          });
          cleanup.push(unpopup);
        }
        if (!alive) return;
        ready = true;
        await tick();
        await syncView();
      } catch (e) {
        report(e);
        if (!tabs.length) {
          tabs = [fresh()];
          active = tabs[0].id;
        }
        ready = true;
      }
    }
    void init();
    const ro = new ResizeObserver(() => {
      if (created.has(active))
        void call("layout", { id: active, ...bounds() }).catch(report);
    });
    ro.observe(viewport);
    return () => {
      alive = false;
      media.removeEventListener("change", mediaChange);
      cleanup.forEach((f) => f());
      ro.disconnect();
    };
  });
</script>

<svelte:window onkeydown={keydown} />
<div class:light={!dark} class="app" style={`--accent:${accent}`}>
  <aside class:collapsed={!vertical}>
    <div class="brand">
      <div class="nova-mark">N</div>
      <span>NOVA<span class="brand-dot">.</span></span><button
        class="icon subtle"
        title="Onglets horizontaux"
        onclick={async () => {
          vertical = false;
          await settingsChanged();
        }}><PanelLeftClose size={17} /></button
      >
    </div>
    <div class="workspace">
      <span class="workspace-symbol">✳</span>
      <div>Espace personnel<small>Un peu plus de calme.</small></div>
      <span class="status-dot"></span>
    </div>
    <div class="section-label">
      VOTRE ESPACE <span>{tabs.length.toString().padStart(2, "0")}</span>
    </div>
    <button class="new-tab" onclick={() => add()}
      ><Plus size={17} /> Nouvel onglet <kbd>⌘ T</kbd></button
    >
    <div class="tab-list">
      {#each tabs as tab (tab.id)}
        <div
          class:active={active === tab.id}
          class="tab"
          draggable="true"
          ondragstart={() => (dragged = tab.id)}
          ondragover={(e) => e.preventDefault()}
          ondrop={() => drop(tab.id)}
          role="group"
        >
          <button class="tab-select" onclick={() => select(tab.id)}
            >{#if tab.loading}<span class="spinner"
              ></span>{:else if tab.url}<Globe2 size={16} />{:else}<span
                class="tiny-nova">✳</span
              >{/if}<span>{tab.title}</span>{#if tab.pinned}<Pin
                size={12}
              />{/if}</button
          >
          <button
            class="tab-close"
            disabled={tab.pinned}
            title="Fermer l’onglet"
            onclick={() => close(tab.id)}><X size={13} /></button
          >
        </div>
      {/each}
    </div>
    <div class="sidebar-bottom">
      <button
        class:chosen={panel === "bookmarks"}
        onclick={() => togglePanel("bookmarks")}
        ><Star size={17} /> Favoris <span>{bookmarks.length}</span></button
      ><button
        class:chosen={panel === "history"}
        onclick={() => togglePanel("history")}
        ><History size={17} /> Historique</button
      >
      <div class="sidebar-divider"></div>
      <button
        class:chosen={panel === "settings"}
        onclick={() => togglePanel("settings")}
        ><SettingsIcon size={17} /> Paramètres <kbd>⌘ ,</kbd></button
      >
      <div class="local-note">
        <ShieldCheck size={13} /> Votre espace. Vos données.
      </div>
    </div>
  </aside>
  <main>
    {#if !vertical}<div class="horizontal-tabs">
        <button
          class="icon"
          title="Onglets verticaux"
          onclick={async () => {
            vertical = true;
            await settingsChanged();
          }}><PanelLeftOpen size={17} /></button
        >{#each tabs as tab}<button
            class:chosen={active === tab.id}
            onclick={() => select(tab.id)}
            >{tab.title}<span
              role="button"
              tabindex="0"
              onclick={(e) => {
                e.stopPropagation();
                void close(tab.id);
              }}
              onkeydown={(e) => {
                if (e.key === "Enter") void close(tab.id);
              }}><X size={12} /></span
            ></button
          >{/each}<button
          class="icon"
          title="Nouvel onglet"
          onclick={() => add()}><Plus size={16} /></button
        >
      </div>{/if}
    <header class="toolbar">
      <div class="nav-controls">
        <button
          class="icon"
          title="Retour"
          disabled={!current?.url}
          onclick={() => doAction("back")}><ArrowLeft size={18} /></button
        ><button
          class="icon"
          title="Suivant"
          disabled={!current?.url}
          onclick={() => doAction("forward")}><ArrowRight size={18} /></button
        ><button
          class="icon"
          title={current?.loading ? "Arrêter" : "Actualiser"}
          disabled={!current?.url}
          onclick={() => doAction(current?.loading ? "stop" : "reload")}
          >{#if current?.loading}<X size={17} />{:else}<RotateCw
              size={16}
            />{/if}</button
        >
      </div>
      <form
        class="address"
        onsubmit={(e) => {
          e.preventDefault();
          void navigate(query);
        }}
      >
        {#if current?.url?.startsWith("https:")}<LockKeyhole
            size={13}
          />{:else}<Search size={15} />{/if}<input
          bind:this={addressInput}
          bind:value={query}
          placeholder="Rechercher ou saisir une adresse"
          aria-label="Barre d’adresse"
          spellcheck="false"
          onfocus={async () => {
            addressing = true;
            await syncView();
          }}
          onblur={async () => {
            addressing = false;
            await syncView();
          }}
        />{#if addressing && suggestions.length}<div class="suggestions">
            {#each suggestions as suggestion}<button
                type="button"
                onpointerdown={(e) => e.preventDefault()}
                onclick={() => navigate(suggestion.url)}
                ><Globe2 size={15} /><span
                  >{suggestion.title}<small>{suggestion.url}</small></span
                ><ArrowUpRight size={14} /></button
              >{/each}
          </div>{/if}<kbd>Ctrl L</kbd><button
          type="button"
          class:starred={isFavorite}
          class="icon"
          title="Ajouter ou retirer des favoris"
          disabled={!current?.url}
          onclick={bookmark}
          ><Star
            size={15}
            fill={isFavorite ? "currentColor" : "none"}
          /></button
        >
      </form>
      <button
        class="icon toolbar-settings"
        title="Paramètres"
        onclick={() => togglePanel("settings")}
        ><SettingsIcon size={18} /></button
      ><button
        class="icon"
        title="Menu de l’onglet"
        onclick={async () => {
          menu = !menu;
          await syncView();
        }}><MoreHorizontal size={21} /></button
      >
    </header>
    <div class="content" bind:this={viewport}>
      {#if menu}<div class="popover">
          <span class="eyebrow">CET ONGLET</span><button
            onclick={() => {
              menu = false;
              void add(current?.url || "");
            }}><Copy size={16} /> Dupliquer</button
          ><button
            onclick={async () => {
              if (current) current.pinned = !current.pinned;
              tabs.sort((a, b) => Number(b.pinned) - Number(a.pinned));
              menu = false;
              await persist();
              await syncView();
            }}
            ><Pin size={16} />
            {current?.pinned ? "Désépingler" : "Épingler"}</button
          ><button
            disabled={!closed.length}
            onclick={() => {
              menu = false;
              void restore();
            }}><History size={16} /> Rouvrir l’onglet fermé</button
          >
          <div class="zoom">
            <button
              onclick={async () => {
                zoom = Math.max(0.5, zoom - 0.1);
                await action(active, "zoom", String(zoom));
              }}>−</button
            ><span>{Math.round(zoom * 100)} %</span><button
              onclick={async () => {
                zoom = Math.min(2, zoom + 0.1);
                await action(active, "zoom", String(zoom));
              }}>+</button
            >
          </div>
          <button onclick={() => togglePanel("settings")}
            ><SettingsIcon size={16} /> Paramètres</button
          >
        </div>{/if}
      {#if panel === "settings"}
        <Settings
          bind:theme
          bind:accent
          bind:vertical
          bind:engine
          onChange={settingsChanged}
          close={() => void togglePanel(null)}
          {clearHistory}
          {clearSites}
        />
      {:else if panel === "history" || panel === "bookmarks"}
        <section class="page">
          <div class="page-top">
            <div>
              <span class="eyebrow">VOTRE COLLECTION</span>
              <h1>
                {panel === "history"
                  ? "Le fil de vos découvertes."
                  : "Gardez l’essentiel."}
              </h1>
              <p>
                {panel === "history"
                  ? "Retrouvez les pages que vous avez parcourues."
                  : "Vos endroits préférés, toujours à portée de main."}
              </p>
            </div>
            <button
              class="icon"
              title="Fermer le panneau"
              onclick={() => togglePanel(null)}><X size={20} /></button
            >
          </div>
          <div class="list-search">
            <Search size={17} /><input
              bind:value={filter}
              placeholder="Rechercher dans cette collection…"
            />
          </div>
          <div class="entries">
            {#each (panel === "history" ? history : bookmarks).filter( (e) => (e.title + e.url)
                  .toLowerCase()
                  .includes(filter.toLowerCase()) ) as entry}<div class="entry">
                <button onclick={() => navigate(entry.url)}
                  ><span class="site-letter"
                    >{host(entry.url)[0].toUpperCase()}</span
                  ><span
                    ><strong>{entry.title}</strong><small>{entry.url}</small
                    ></span
                  ><ArrowUpRight size={17} /></button
                >{#if panel === "bookmarks"}<button
                    class="icon"
                    title="Supprimer le favori"
                    onclick={() => removeBookmark(entry.url)}
                    ><Trash2 size={15} /></button
                  >{/if}
              </div>{:else}<div class="empty">
                <Globe2 size={30} />
                <h2>Tout commence ici.</h2>
                <p>
                  {filter
                    ? "Aucun résultat pour cette recherche."
                    : panel === "history"
                      ? "Votre historique apparaîtra après vos premières visites."
                      : "Ajoutez une page avec l’étoile de la barre d’adresse."}
                </p>
              </div>{/each}
          </div>
        </section>
      {:else if !current?.url}
        <Home
          {bookmarks}
          {engine}
          {navigate}
          openBookmarks={() => void togglePanel("bookmarks")}
        />
      {:else}<div class="native-placeholder">
          <Globe2 size={28} />
          <p>
            {native
              ? "La page est affichée dans une WebView native."
              : "La navigation est disponible dans l’application desktop."}
          </p>
          {#if current.loading}<span class="spinner"></span>{/if}
        </div>{/if}
    </div>
    {#if error}<div class="toast" role="alert">
        <span>{error}</span><button
          class="icon"
          title="Fermer le message"
          onclick={async () => {
            error = "";
            await syncView();
          }}><X size={16} /></button
        >
      </div>{/if}
  </main>
</div>
