<script lang="ts">
  import { onMount } from "svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
  import * as api from "$lib/api";
  import { lang, LANGUAGES, t, type Lang } from "$lib/i18n";
  import { clockTime } from "$lib/format";
  import KeyField from "$lib/KeyField.svelte";
  import PageHeader from "$lib/PageHeader.svelte";
  import type { KeyInfo, Settings, StatusView } from "$lib/types";

  const INTERVALS = [1, 2, 5, 10, 15, 30];
  const SIZES = [
    { label: "S", width: 220, height: 56 },
    { label: "M", width: 280, height: 80 },
    { label: "L", width: 360, height: 110 },
  ];

  let settings = $state<Settings | null>(null);
  let status = $state<StatusView | null>(null);
  let keyInfo = $state<KeyInfo | null>(null);
  let autostart = $state(false);
  let editingKey = $state(false);
  let version = $state("");

  const activeSize = $derived(
    SIZES.find((s) => s.width === settings?.widgetWidth && s.height === settings?.widgetHeight)?.label,
  );

  async function update(patch: Partial<Settings>) {
    if (!settings) return;
    settings = await api.saveSettings({ ...settings, ...patch });
  }

  async function toggleAutostart(enabled: boolean) {
    try {
      if (enabled) await enable();
      else await disable();
    } finally {
      autostart = await isEnabled();
    }
  }

  async function onKeySaved() {
    editingKey = false;
    keyInfo = await api.getKeyInfo();
  }

  onMount(() => {
    (async () => {
      settings = await api.getSettings();
      keyInfo = await api.getKeyInfo();
      autostart = await isEnabled().catch(() => false);
      version = await getVersion();
    })();
    const unlisten = api.subscribeStatus((s) => (status = s));
    return () => {
      unlisten.then((fn) => fn());
    };
  });
</script>

<main class="page">
  <PageHeader title={t.settings} intro={t.settingsIntro} />

  {#if settings && keyInfo}
    <section>
      <h2>{t.account}</h2>
      {#if editingKey}
        <KeyField onSaved={onKeySaved} onCancel={() => (editingKey = false)} />
      {:else}
        <div class="line">
          <span class="label">{t.apiKey}</span>
          <code>{keyInfo.maskedKey}</code>
          <span class="spacer"></span>
          <button class="btn" onclick={() => (editingKey = true)}>{t.changeKey}</button>
          <button class="btn danger" onclick={() => api.removeKey()}>{t.signOut}</button>
        </div>
      {/if}
      {#if status}
        <div class="line">
          <span class="label">{t.lastCheck}</span>
          <span>{status.checking ? t.checking : (clockTime(status.lastCheck) ?? t.never)}</span>
          <span class="spacer"></span>
          <button class="btn" onclick={() => api.checkNow()} disabled={status.checking}>{t.checkNow}</button>
          <button class="btn primary" onclick={() => api.openDashboard()}>{t.dashboard} ↗</button>
        </div>
        {#if status.mutedUntil}
          <div class="line">
            <span class="label">{t.mutedUntil}</span>
            <span>{clockTime(status.mutedUntil)}</span>
            <span class="spacer"></span>
            <button class="btn" onclick={() => api.unmute()}>{t.unmute}</button>
          </div>
        {/if}
      {/if}
    </section>

    <section>
      <h2>{t.monitoring}</h2>
      <label class="line">
        <span class="label">{t.interval}</span>
        <select
          class="field"
          value={settings.intervalMinutes}
          onchange={(e) => update({ intervalMinutes: Number(e.currentTarget.value) })}
        >
          {#each INTERVALS as minutes (minutes)}
            <option value={minutes}>{t.everyMinutes(minutes)}</option>
          {/each}
        </select>
      </label>
      <label class="check">
        <input
          type="checkbox"
          checked={settings.countLooksDown}
          onchange={(e) => update({ countLooksDown: e.currentTarget.checked })}
        />
        <span>{t.countLooksDown}</span>
      </label>
    </section>

    <section>
      <h2>{t.widget}</h2>
      <div class="line">
        <span class="label">{t.size}</span>
        <div class="segmented" role="group" aria-label={t.size}>
          {#each SIZES as size (size.label)}
            <button
              class:active={activeSize === size.label}
              aria-pressed={activeSize === size.label}
              onclick={() => update({ widgetWidth: size.width, widgetHeight: size.height })}>{size.label}</button
            >
          {/each}
        </div>
      </div>
      <p class="hint">{t.sizeHint}</p>
      <label class="check">
        <input
          type="checkbox"
          checked={settings.showWhenOk}
          onchange={(e) => update({ showWhenOk: e.currentTarget.checked })}
        />
        <span>{t.showWhenOk}</span>
      </label>
    </section>

    <section>
      <h2>{t.general}</h2>
      <label class="line">
        <span class="label">{t.language}</span>
        <select class="field" value={lang} onchange={(e) => update({ language: e.currentTarget.value as Lang })}>
          {#each LANGUAGES as option (option.code)}
            <option value={option.code}>{option.name}</option>
          {/each}
        </select>
      </label>
      <label class="check">
        <input type="checkbox" checked={autostart} onchange={(e) => toggleAutostart(e.currentTarget.checked)} />
        <span>{t.autostart}</span>
      </label>
    </section>

    <footer class="page-footer">
      <p>UptimeWidget {version} · {t.storedSecurely}</p>
    </footer>
  {/if}
</main>

<style>
  section {
    padding: 4px 0 14px;
  }
  section + section {
    border-top: 1px solid var(--border);
    padding-top: 14px;
  }

  h2 {
    font-size: 15px;
    font-weight: 600;
    margin: 0 0 8px;
  }

  .line {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 40px;
  }
  .label {
    width: 112px;
    flex: none;
    color: var(--text-muted);
  }
  .spacer {
    flex: 1;
  }

  code {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 12px;
  }

  .segmented {
    display: inline-flex;
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
  }
  .segmented button {
    border: 0;
    background: var(--surface);
    color: var(--text);
    font-weight: 600;
    min-width: 44px;
    padding: 7px 12px;
  }
  .segmented button + button {
    border-left: 1px solid var(--border);
  }
  .segmented button.active {
    background: var(--accent);
    color: var(--accent-text);
  }

  .check {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 6px 0;
  }
  .check input {
    margin-top: 3px;
    flex: none;
  }

  .hint {
    margin: 2px 0 6px;
    font-size: 12px;
    color: var(--text-muted);
  }
</style>
