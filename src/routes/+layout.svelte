<script lang="ts">
  import "../app.css";
  import { initI18n } from "$lib/i18n";

  let { children } = $props();

  // Strings must be loaded before any page renders.
  const ready = initI18n();
</script>

{#await ready then}
  {@render children()}
{/await}
