<script lang="ts">
  import { X, ShieldCheck, Check } from "lucide-svelte";
  import type { Engine } from "./core";
  let {
    theme = $bindable(),
    accent = $bindable(),
    vertical = $bindable(),
    engine = $bindable(),
    onChange,
    close,
    clearHistory,
    clearSites,
  }: {
    theme: string;
    accent: string;
    vertical: boolean;
    engine: Engine;
    onChange: () => Promise<void>;
    close: () => void;
    clearHistory: () => Promise<void>;
    clearSites: () => Promise<void>;
  } = $props();
</script>

<section class="page settings-page">
  <div class="page-top">
    <div>
      <span class="eyebrow">À VOTRE IMAGE</span>
      <h1>Les petits détails comptent.</h1>
      <p>Un navigateur qui s’adapte à vous.</p>
    </div>
    <button class="icon" title="Fermer les paramètres" onclick={close}
      ><X size={20} /></button
    >
  </div>
  <h2>Apparence</h2>
  <div class="setting-card">
    <div class="setting-row">
      <div>
        <strong>Un autre éclairage</strong>
        <p>Choisissez l’ambiance de votre espace.</p>
      </div>
      <div class="segmented">
        {#each [{ id: "light", label: "Clair" }, { id: "dark", label: "Sombre" }, { id: "auto", label: "Auto" }] as t}<button
            class:selected={theme === t.id}
            onclick={async () => {
              theme = t.id;
              await onChange();
            }}>{t.label}</button
          >{/each}
      </div>
    </div>
    <div class="setting-row">
      <div>
        <strong>Votre touche de couleur</strong>
        <p>Une nuance discrète, partout dans NOVA.</p>
      </div>
      <div class="swatches">
        {#each ["#b8c5a4", "#aabcf0", "#d6b2cc", "#e0bb90", "#91c9c6"] as color}<button
            style={`background:${color}`}
            title={color}
            onclick={async () => {
              accent = color;
              await onChange();
            }}
            >{#if accent === color}<Check size={15} />{/if}</button
          >{/each}
      </div>
    </div>
    <div class="setting-row">
      <div>
        <strong>Disposition des onglets</strong>
        <p>Votre façon de parcourir le web.</p>
      </div>
      <select bind:value={vertical} onchange={onChange}
        ><option value={true}>Verticaux</option><option value={false}
          >Horizontaux</option
        ></select
      >
    </div>
  </div>
  <h2>Recherche</h2>
  <div class="setting-card setting-row">
    <div>
      <strong>Moteur de recherche</strong>
      <p>Utilisé pour les recherches dans la barre d’adresse.</p>
    </div>
    <select bind:value={engine} onchange={onChange}
      ><option value="duckduckgo">DuckDuckGo</option><option value="google"
        >Google</option
      ><option value="bing">Bing</option><option value="brave"
        >Brave Search</option
      ></select
    >
  </div>
  <h2>Permissions</h2>
  <div class="setting-card setting-row">
    <div>
      <strong>Accès aux appareils et notifications</strong>
      <p>
        Les demandes gérées par Tauri sont refusées dans ce MVP. Le contrôle par
        site sera ajouté après validation sur chaque plateforme.
      </p>
    </div>
    <ShieldCheck size={20} />
  </div>
  <h2>Données locales</h2>
  <div class="setting-card">
    <div class="setting-row">
      <div>
        <strong>Historique de navigation</strong>
        <p>Supprimer les adresses enregistrées dans NOVA.</p>
      </div>
      <button class="secondary" onclick={clearHistory}
        >Effacer l’historique</button
      >
    </div>
    <div class="setting-row">
      <div>
        <strong>Cookies et données des sites</strong>
        <p>
          Concerne les WebViews actuellement ouvertes. Vous serez déconnecté.
        </p>
      </div>
      <button class="secondary" onclick={clearSites}>Effacer</button>
    </div>
  </div>
  <div class="about">
    <span class="tiny-nova">✳</span>
    <div>
      NOVA Browser <span>0.1.0 · MVP</span>
      <p>Tauri · WebView natif · Aucune télémétrie ajoutée</p>
    </div>
  </div>
</section>
