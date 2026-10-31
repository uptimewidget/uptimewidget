<script lang="ts">
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import * as api from "$lib/api";
  import { t } from "$lib/i18n";
  import PageHeader from "$lib/PageHeader.svelte";

  let version = $state("");

  type UpdateState =
    | { kind: "idle" }
    | { kind: "checking" }
    | { kind: "upToDate" }
    | { kind: "available"; version: string }
    | { kind: "installing" }
    | { kind: "failed" };
  let update = $state<UpdateState>({ kind: "idle" });

  async function checkForUpdate() {
    update = { kind: "checking" };
    try {
      const result = await api.checkForUpdate();
      update = result.status === "available" ? { kind: "available", version: result.version } : { kind: "upToDate" };
    } catch {
      update = { kind: "failed" };
    }
  }

  async function installUpdate() {
    update = { kind: "installing" };
    try {
      await api.installUpdate();
    } catch {
      update = { kind: "failed" };
    }
  }

  onMount(async () => {
    version = await getVersion();
  });
</script>

<main class="page">
  <PageHeader title="UptimeWidget" intro={t.tagline} />

  <div class="meta">
    <dl>
      <dt>{t.version}</dt>
      <dd>{version}</dd>
    </dl>
    <button class="github" onclick={() => api.openRepository()} title="GitHub" aria-label="GitHub">
      <svg viewBox="0 0 16 16" aria-hidden="true">
        <path
          d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"
        />
      </svg>
    </button>
  </div>

  <div class="update">
    {#if update.kind === "available"}
      <span>{t.updateAvailable.replace("{v}", update.version)}</span>
      <button class="btn primary" onclick={installUpdate}>{t.installUpdate}</button>
    {:else if update.kind === "installing"}
      <span>{t.installingUpdate}</span>
    {:else}
      <span class="muted">
        {#if update.kind === "upToDate"}{t.upToDate}{:else if update.kind === "failed"}{t.updateFailed}{/if}
      </span>
      <button class="btn" onclick={checkForUpdate} disabled={update.kind === "checking"}>
        {update.kind === "checking" ? t.checkingUpdates : t.checkUpdates}
      </button>
    {/if}
  </div>

  <button class="author" onclick={() => api.openAuthorWebsite()}>
    <span>{t.madeBy} <strong>Peter Bagi</strong></span>
    <span class="pitch">{t.authorPitch}</span>
    <span class="link">peterbagi.de ↗</span>
  </button>

  <footer class="page-footer">
    <p>© 2026 Peter Bagi · {t.license}</p>
    <p>{t.disclaimer}</p>
  </footer>
</main>

<style>
  .meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  dl {
    display: flex;
    gap: 8px;
    margin: 0;
  }
  dt {
    color: var(--text-muted);
  }
  dd {
    margin: 0;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .update {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-top: 16px;
    min-height: 36px;
  }
  .muted {
    color: var(--text-muted);
  }

  .author {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    margin-top: 16px;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    color: var(--text);
    text-align: left;
  }
  .author:hover {
    background: var(--bg-subtle);
  }
  .pitch {
    color: var(--text-muted);
    font-size: 13px;
  }
  .link {
    color: var(--accent);
    font-size: 13px;
  }

  .github {
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    color: var(--text);
  }
  .github:hover {
    background: var(--bg-subtle);
  }
  .github svg {
    width: 20px;
    height: 20px;
    fill: currentColor;
  }
</style>
