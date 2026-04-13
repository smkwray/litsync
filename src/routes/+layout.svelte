<script lang="ts">
	import '../app.css';
	const favicon = '/favicon.png';
	import PlayerBar from '$lib/components/PlayerBar.svelte';
	import ShortcutOverlay from '$lib/components/ShortcutOverlay.svelte';
	import SettingsModal from '$lib/components/SettingsModal.svelte';
	import { playerState } from '$lib/stores/player';
	import { ensureSettingsLoaded } from '$lib/stores/settings';
	import { afterNavigate } from '$app/navigation';
	import { onMount } from 'svelte';

	onMount(() => ensureSettingsLoaded());

	let { children } = $props();
	let showShortcuts = $state(false);
	let showSettings = $state(false);

	// Restore keyboard focus after every client-side navigation
	afterNavigate(() => {
		// Small delay to let the new page render
		requestAnimationFrame(() => document.body.focus());
	});

	function handleGlobalKeydown(e: KeyboardEvent) {
		if (e.defaultPrevented) return;
		const target = e.target as HTMLElement;
		const inInput = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable;

		if (e.key === ',' && (e.metaKey || e.ctrlKey)) {
			e.preventDefault();
			showSettings = !showSettings;
			return;
		}

		if (inInput) return;

		if (e.key === '?' || (e.shiftKey && e.key === '/')) {
			e.preventDefault();
			showShortcuts = !showShortcuts;
		}
	}
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div class={$playerState.isLoaded ? 'pb-20' : ''}>
	{@render children()}
</div>

<PlayerBar />
<ShortcutOverlay bind:open={showShortcuts} />
<SettingsModal bind:open={showSettings} />
