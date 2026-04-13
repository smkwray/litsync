<script lang="ts">
	import { settingsStore, updateSettings } from '$lib/stores/settings';
	import { toggleSilenceSkip, playerState } from '$lib/stores/player';
	import { getDataDir, getAppMode } from '$lib/api/tauri';
	import { onMount } from 'svelte';
	import type { AppTheme } from '$lib/types/book';

	let { open = $bindable(false) } = $props();
	let dataDir = $state('');
	let appMode = $state('');

	onMount(async () => {
		try {
			dataDir = await getDataDir();
			appMode = await getAppMode();
		} catch {}
	});

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); open = false; }
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) open = false;
	}

	function setTheme(theme: AppTheme) {
		updateSettings({ theme });
	}

	let firstFocusable = $state<HTMLElement>();
	$effect(() => { if (open && firstFocusable) firstFocusable.focus(); });
</script>

{#if open}
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div
		class="fixed inset-0 z-[100] flex items-center justify-center bg-black/60 backdrop-blur-sm"
		onkeydown={handleKeydown}
		onclick={handleBackdropClick}
		role="dialog"
		tabindex="-1"
		aria-label="Settings"
		aria-modal="true"
	>
		<div class="mx-4 max-h-[80vh] w-full max-w-lg overflow-y-auto rounded-xl bg-zinc-800 shadow-2xl">
			<div class="flex items-center justify-between border-b border-zinc-700 px-6 py-4">
				<h2 class="text-lg font-semibold text-zinc-100">Settings</h2>
				<button
					bind:this={firstFocusable}
					onclick={() => (open = false)}
					class="rounded p-1 text-zinc-400 transition-colors hover:bg-zinc-700 hover:text-zinc-200 focus-visible:text-zinc-200"
					aria-label="Close"
				>
					<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
						<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</button>
			</div>

			<div class="space-y-8 px-6 py-5">
				<section>
					<h3 class="mb-4 text-xs font-semibold uppercase tracking-wider text-zinc-400">Appearance</h3>
					<div class="flex items-center justify-between gap-4">
						<div>
							<p class="text-sm text-zinc-200">Theme</p>
							<p class="text-xs text-zinc-500">Controls the app colors and native macOS title bar.</p>
						</div>
						<div class="inline-flex rounded-xl bg-zinc-700 p-1">
							<button
								onclick={() => setTheme('dark')}
								class="rounded-lg px-3 py-1.5 text-sm transition-colors {$settingsStore.theme === 'dark' ? 'bg-zinc-900 text-zinc-100 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
								aria-pressed={$settingsStore.theme === 'dark'}
							>
								Dark
							</button>
							<button
								onclick={() => setTheme('light')}
								class="rounded-lg px-3 py-1.5 text-sm transition-colors {$settingsStore.theme === 'light' ? 'bg-zinc-100 text-zinc-900 shadow-sm' : 'text-zinc-400 hover:text-zinc-200'}"
								aria-pressed={$settingsStore.theme === 'light'}
							>
								Light
							</button>
						</div>
					</div>
				</section>

				<!-- Playback -->
				<section>
					<h3 class="mb-4 text-xs font-semibold uppercase tracking-wider text-zinc-400">Playback</h3>
					<div class="space-y-4">
						<div>
							<div class="mb-1 flex items-center justify-between">
								<label for="default-speed" class="text-sm text-zinc-300">Default speed</label>
								<span class="font-mono text-xs text-zinc-500">{$settingsStore.default_speed.toFixed(2)}x</span>
							</div>
							<input id="default-speed" type="range" min="0.5" max="3" step="0.05"
								value={$settingsStore.default_speed}
								oninput={(e) => updateSettings({ default_speed: Number((e.target as HTMLInputElement).value) })}
								class="h-1.5 w-full cursor-pointer appearance-none rounded-full bg-zinc-600 accent-indigo-500" />
						</div>

						<div class="grid grid-cols-2 gap-4">
							<div>
								<div class="mb-1 flex items-center justify-between">
									<label for="skip-short" class="text-sm text-zinc-300">Short skip</label>
									<span class="font-mono text-xs text-zinc-500">{$settingsStore.skip_short_sec}s</span>
								</div>
								<input id="skip-short" type="range" min="5" max="60" step="5"
									value={$settingsStore.skip_short_sec}
									oninput={(e) => updateSettings({ skip_short_sec: Number((e.target as HTMLInputElement).value) })}
									class="h-1.5 w-full cursor-pointer appearance-none rounded-full bg-zinc-600 accent-indigo-500" />
							</div>
							<div>
								<div class="mb-1 flex items-center justify-between">
									<label for="skip-long" class="text-sm text-zinc-300">Long skip</label>
									<span class="font-mono text-xs text-zinc-500">{$settingsStore.skip_long_sec}s</span>
								</div>
								<input id="skip-long" type="range" min="10" max="120" step="5"
									value={$settingsStore.skip_long_sec}
									oninput={(e) => updateSettings({ skip_long_sec: Number((e.target as HTMLInputElement).value) })}
									class="h-1.5 w-full cursor-pointer appearance-none rounded-full bg-zinc-600 accent-indigo-500" />
							</div>
						</div>
					</div>
				</section>

				<!-- Auto-rewind -->
				<section>
					<h3 class="mb-4 text-xs font-semibold uppercase tracking-wider text-zinc-400">Auto-rewind after pause</h3>
					<div class="space-y-4">
						<div class="flex items-center justify-between">
							<div>
								<p class="text-sm text-zinc-200">Enabled</p>
								<p class="text-xs text-zinc-500">Rewind a few seconds when resuming after a pause</p>
							</div>
							<button
								onclick={() => updateSettings({ auto_rewind_enabled: !$settingsStore.auto_rewind_enabled })}
								class="relative h-6 w-11 rounded-full transition-colors {$settingsStore.auto_rewind_enabled ? 'bg-indigo-600' : 'bg-zinc-600'}"
								role="switch" aria-checked={$settingsStore.auto_rewind_enabled} aria-label="Toggle auto-rewind"
							>
								<span class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform {$settingsStore.auto_rewind_enabled ? 'translate-x-5' : ''}"></span>
							</button>
						</div>

						{#if $settingsStore.auto_rewind_enabled}
							<div class="grid grid-cols-2 gap-4">
								<div>
									<p class="mb-1 text-xs text-zinc-400">Short pause (&gt;{$settingsStore.auto_rewind_short_pause_sec}s)</p>
									<div class="flex items-center gap-2">
										<span class="text-xs text-zinc-500">Rewind</span>
										<input type="number" min="1" max="30"
											value={$settingsStore.auto_rewind_short_rewind_sec}
											onchange={(e) => updateSettings({ auto_rewind_short_rewind_sec: Number((e.target as HTMLInputElement).value) })}
											class="w-14 rounded bg-zinc-700 px-2 py-1 text-xs text-zinc-200" />
										<span class="text-xs text-zinc-500">s</span>
									</div>
								</div>
								<div>
									<p class="mb-1 text-xs text-zinc-400">Long pause (&gt;{$settingsStore.auto_rewind_long_pause_sec / 60}min)</p>
									<div class="flex items-center gap-2">
										<span class="text-xs text-zinc-500">Rewind</span>
										<input type="number" min="5" max="120"
											value={$settingsStore.auto_rewind_long_rewind_sec}
											onchange={(e) => updateSettings({ auto_rewind_long_rewind_sec: Number((e.target as HTMLInputElement).value) })}
											class="w-14 rounded bg-zinc-700 px-2 py-1 text-xs text-zinc-200" />
										<span class="text-xs text-zinc-500">s</span>
									</div>
								</div>
							</div>
						{/if}
					</div>
				</section>

				<!-- Silence Skip -->
				<section>
					<h3 class="mb-4 text-xs font-semibold uppercase tracking-wider text-zinc-400">Silence Skip</h3>
					<div class="space-y-4">
						<div class="flex items-center justify-between">
							<div>
								<p class="text-sm text-zinc-200">Skip blank spaces</p>
								<p class="text-xs text-zinc-500">Pre-analyzes audio and skips silence during playback</p>
							</div>
							<button
								onclick={() => toggleSilenceSkip()}
								class="relative h-6 w-11 rounded-full transition-colors {$playerState.silenceSkip ? 'bg-indigo-600' : 'bg-zinc-600'}"
								role="switch" aria-checked={$playerState.silenceSkip} aria-label="Toggle silence skip"
							>
								<span class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform {$playerState.silenceSkip ? 'translate-x-5' : ''}"></span>
							</button>
						</div>

						<div>
							<div class="mb-1 flex items-center justify-between">
								<label for="silence-threshold" class="text-sm text-zinc-300">Sensitivity</label>
								<span class="font-mono text-xs text-zinc-500">{$settingsStore.silence_skip_threshold} dBFS</span>
							</div>
							<input id="silence-threshold" type="range" min="-70" max="-15" step="1"
								value={$settingsStore.silence_skip_threshold}
								oninput={(e) => updateSettings({ silence_skip_threshold: Number((e.target as HTMLInputElement).value) })}
								class="h-1.5 w-full cursor-pointer appearance-none rounded-full bg-zinc-600 accent-indigo-500" />
							<div class="mt-1 flex justify-between text-[10px] text-zinc-600">
								<span>Less aggressive</span>
								<span>More aggressive</span>
							</div>
						</div>

						<div>
							<div class="mb-1 flex items-center justify-between">
								<label for="silence-gap" class="text-sm text-zinc-300">Minimum gap</label>
								<span class="font-mono text-xs text-zinc-500">{$settingsStore.silence_skip_min_gap_ms}ms</span>
							</div>
							<input id="silence-gap" type="range" min="100" max="3000" step="100"
								value={$settingsStore.silence_skip_min_gap_ms}
								oninput={(e) => updateSettings({ silence_skip_min_gap_ms: Number((e.target as HTMLInputElement).value) })}
								class="h-1.5 w-full cursor-pointer appearance-none rounded-full bg-zinc-600 accent-indigo-500" />
						</div>
					</div>
				</section>

				<!-- Sleep Timer -->
				<section>
					<h3 class="mb-4 text-xs font-semibold uppercase tracking-wider text-zinc-400">Sleep Timer</h3>
					<div>
						<div class="mb-1 flex items-center justify-between">
							<label for="sleep-default" class="text-sm text-zinc-300">Default duration</label>
							<span class="font-mono text-xs text-zinc-500">{$settingsStore.sleep_timer_default_min} min</span>
						</div>
						<input id="sleep-default" type="range" min="5" max="120" step="5"
							value={$settingsStore.sleep_timer_default_min}
							oninput={(e) => updateSettings({ sleep_timer_default_min: Number((e.target as HTMLInputElement).value) })}
							class="h-1.5 w-full cursor-pointer appearance-none rounded-full bg-zinc-600 accent-indigo-500" />
					</div>
				</section>

				<!-- Info -->
				<section>
					<h3 class="mb-4 text-xs font-semibold uppercase tracking-wider text-zinc-400">Info</h3>
					<div class="space-y-2 text-xs text-zinc-500">
						<div class="flex justify-between">
							<span>Mode</span>
							<span class="text-zinc-400">{appMode}</span>
						</div>
						<div class="flex justify-between">
							<span>Data directory</span>
							<span class="max-w-[250px] truncate text-zinc-400" title={dataDir}>{dataDir}</span>
						</div>
						<div class="flex justify-between">
							<span>SABP sync</span>
							<span class="text-indigo-400">Active</span>
						</div>
						<div class="mt-2">
							<p>LitSync v0.1.0</p>
							<p>Press <kbd class="rounded bg-zinc-700 px-1 py-0.5">?</kbd> for keyboard shortcuts</p>
						</div>
					</div>
				</section>
			</div>
		</div>
	</div>
{/if}
