<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { readText } from "@tauri-apps/plugin-clipboard-manager";
  import * as api from "$lib/api";
  import { errorMessage, t } from "$lib/i18n";
  import type { AccountInfo } from "$lib/types";

  interface Props {
    /** Pick up a copied key from the clipboard whenever the window gains focus. */
    autoDetect?: boolean;
    onSaved: (info: AccountInfo) => void;
    onCancel?: () => void;
  }

  let { autoDetect = false, onSaved, onCancel }: Props = $props();

  // UptimeRobot keys start with "ur" followed by letters, digits and dashes.
  const KEY_PATTERN = /^ur[A-Za-z0-9-]{16,120}$/;

  let key = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);
  let detected = $state(false);
  let lastClipboard = "";

  async function submit() {
    const value = key.trim();
    if (!value || busy) return;
    busy = true;
    error = null;
    try {
      onSaved(await api.saveKey(value));
    } catch (code) {
      error = errorMessage(code);
      detected = false;
    } finally {
      busy = false;
    }
  }

  async function checkClipboard() {
    if (busy) return;
    let text: string;
    try {
      text = (await readText()).trim();
    } catch {
      return;
    }
    if (text === lastClipboard || !KEY_PATTERN.test(text) || text === key.trim()) return;
    lastClipboard = text;
    key = text;
    detected = true;
    await submit();
  }

  onMount(() => {
    if (!autoDetect) return;
    checkClipboard();
    const unlisten = getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (focused) checkClipboard();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  });
</script>

<form
  onsubmit={(event) => {
    event.preventDefault();
    submit();
  }}
>
  <div class="row">
    <input
      class="field"
      type="password"
      bind:value={key}
      placeholder={t.keyPlaceholder}
      autocomplete="off"
      spellcheck="false"
      aria-invalid={error !== null}
      oninput={() => {
        error = null;
        detected = false;
      }}
    />
    <button class="btn primary" type="submit" disabled={busy || !key.trim()}>
      {busy ? t.connecting : t.connect}
    </button>
    {#if onCancel}
      <button class="btn" type="button" onclick={onCancel}>{t.cancel}</button>
    {/if}
  </div>
  {#if error}
    <p class="message error" role="alert">{error}</p>
  {:else if detected && busy}
    <p class="message">{t.detectedFromClipboard} …</p>
  {/if}
</form>

<style>
  .row {
    display: flex;
    gap: 8px;
  }

  input {
    flex: 1;
    min-width: 0;
  }
  input[aria-invalid="true"] {
    border-color: var(--danger);
  }

  .message {
    margin: 8px 0 0;
    font-size: 12px;
    color: var(--text-muted);
  }
  .message.error {
    color: var(--danger);
  }
</style>
