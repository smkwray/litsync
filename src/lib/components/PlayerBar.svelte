<script lang="ts">
	import {
		playerState,
		currentChapter,
		toggle,
		seekRelative,
		nextChapter,
		prevChapter,
		adjustSpeed,
		setSpeed,
		setVolume,
		seekTo,
		toggleSilenceSkip,
	} from '$lib/stores/player';
	import { assetUrl } from '$lib/api/tauri';
	import { settingsStore } from '$lib/stores/settings';
	import { formatTime } from '$lib/utils/time';

	let dragging = $state(false);
	let dragValue = $state(0);

	function onSeekInput(e: Event) {
		const input = e.target as HTMLInputElement;
		dragValue = Number(input.value);
		dragging = true;
	}

	function onSeekChange(e: Event) {
		const input = e.target as HTMLInputElement;
		seekTo(Number(input.value));
		dragging = false;
	}

	function handleKeydown(e: KeyboardEvent) {
		const target = e.target as HTMLElement;
		if (target.tagName === 'INPUT' && target.getAttribute('type') === 'range') return;

		const shortMs = $settingsStore.skip_short_sec * 1000;
		const longMs = $settingsStore.skip_long_sec * 1000;

		switch (e.key) {
			case ' ':
				e.preventDefault();
				toggle();
				break;
			case 'ArrowLeft':
				e.preventDefault();
				seekRelative(e.shiftKey ? -longMs : -shortMs);
				break;
			case 'ArrowRight':
				e.preventDefault();
				seekRelative(e.shiftKey ? longMs : shortMs);
				break;
			case 'n':
				e.preventDefault();
				nextChapter();
				break;
			case 'p':
				e.preventDefault();
				prevChapter();
				break;
			case '[':
				e.preventDefault();
				adjustSpeed(-0.1);
				break;
			case ']':
				e.preventDefault();
				adjustSpeed(0.1);
				break;
			case 's':
				e.preventDefault();
				toggleSilenceSkip();
				break;
		}
	}

	const displayTime = $derived(dragging ? dragValue : $playerState.currentTimeMs);
</script>

<svelte:window onkeydown={(e) => {
	if (e.defaultPrevented) return;
	const target = e.target as HTMLElement;
	if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) return;
	if (!$playerState.isLoaded) return;
	handleKeydown(e);
}} />

{#if $playerState.isLoaded}
	<div class="fixed bottom-0 left-0 right-0 z-50 border-t border-zinc-700 bg-zinc-900/95 backdrop-blur-sm">
		<!-- Seek bar - full width at top of bar -->
		<div class="px-4 pt-1">
			<input
				type="range"
				min="0"
				max={$playerState.totalDurationMs}
				value={displayTime}
				oninput={onSeekInput}
				onchange={onSeekChange}
				class="h-1 w-full cursor-pointer appearance-none rounded-full bg-zinc-700 accent-indigo-500"
				aria-label="Seek position"
			/>
		</div>

		<div class="flex items-center gap-4 px-4 py-2">
			<!-- Cover + info -->
			<div class="flex min-w-0 flex-1 items-center gap-3">
				<div class="h-10 w-10 flex-shrink-0 overflow-hidden rounded bg-zinc-800">
					{#if $playerState.coverPath}
						<img src={assetUrl($playerState.coverPath)} alt="" class="h-full w-full object-cover" />
					{:else}
						<div class="flex h-full w-full items-center justify-center bg-zinc-700 text-sm font-bold text-zinc-500">
							{$playerState.bookTitle.charAt(0)}
						</div>
					{/if}
				</div>
				<div class="min-w-0">
					<p class="truncate text-sm font-medium text-zinc-100">{$playerState.bookTitle}</p>
					<p class="truncate text-xs text-zinc-400">{$currentChapter?.title ?? ''}</p>
				</div>
			</div>

			<!-- Transport controls -->
			<div class="flex items-center gap-1">
				<!-- Prev chapter -->
				<button
					onclick={() => prevChapter()}
					class="rounded p-1.5 text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-zinc-200 focus-visible:text-zinc-200"
					title="Previous chapter (P)"
					aria-label="Previous chapter"
				>
					<svg class="h-4 w-4" fill="currentColor" viewBox="0 0 24 24">
						<path d="M6 6h2v12H6V6zm3.5 6l8.5 6V6l-8.5 6z"/>
					</svg>
				</button>

				<!-- Rewind 10s -->
				<button
					onclick={() => seekRelative(-10000)}
					class="rounded p-1.5 text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-zinc-200 focus-visible:text-zinc-200"
					title="Rewind 10s (Left)"
					aria-label="Rewind 10 seconds"
				>
					<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
						<path stroke-linecap="round" stroke-linejoin="round" d="M12.066 11.2a1 1 0 000 1.6l5.334 4A1 1 0 0019 16V8a1 1 0 00-1.6-.8l-5.334 4zM4.066 11.2a1 1 0 000 1.6l5.334 4A1 1 0 0011 16V8a1 1 0 00-1.6-.8l-5.334 4z"/>
					</svg>
				</button>

				<!-- Play/Pause -->
				<button
					onclick={() => toggle()}
					class="rounded-full bg-zinc-100 p-2 text-zinc-900 transition-colors hover:bg-white focus-visible:ring-2 focus-visible:ring-indigo-400"
					title={$playerState.isPlaying ? 'Pause (Space)' : 'Play (Space)'}
					aria-label={$playerState.isPlaying ? 'Pause' : 'Play'}
				>
					{#if $playerState.isPlaying}
						<svg class="h-5 w-5" fill="currentColor" viewBox="0 0 24 24">
							<path d="M6 4h4v16H6V4zm8 0h4v16h-4V4z"/>
						</svg>
					{:else}
						<svg class="h-5 w-5" fill="currentColor" viewBox="0 0 24 24">
							<path d="M8 5v14l11-7z"/>
						</svg>
					{/if}
				</button>

				<!-- Forward 10s -->
				<button
					onclick={() => seekRelative(10000)}
					class="rounded p-1.5 text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-zinc-200 focus-visible:text-zinc-200"
					title="Forward 10s (Right)"
					aria-label="Forward 10 seconds"
				>
					<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
						<path stroke-linecap="round" stroke-linejoin="round" d="M11.933 12.8a1 1 0 000-1.6L6.6 7.2A1 1 0 005 8v8a1 1 0 001.6.8l5.333-4zM19.933 12.8a1 1 0 000-1.6l-5.333-4A1 1 0 0013 8v8a1 1 0 001.6.8l5.333-4z"/>
					</svg>
				</button>

				<!-- Next chapter -->
				<button
					onclick={() => nextChapter()}
					class="rounded p-1.5 text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-zinc-200 focus-visible:text-zinc-200"
					title="Next chapter (N)"
					aria-label="Next chapter"
				>
					<svg class="h-4 w-4" fill="currentColor" viewBox="0 0 24 24">
						<path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z"/>
					</svg>
				</button>
			</div>

			<!-- Right side: time, speed, volume -->
			<div class="flex items-center gap-3 text-xs text-zinc-400">
				<span class="font-mono tabular-nums">
					{formatTime(displayTime)} / {formatTime($playerState.totalDurationMs)}
				</span>

				<!-- Silence skip -->
				<button
					onclick={() => toggleSilenceSkip()}
					class="rounded px-1.5 py-0.5 text-xs transition-colors hover:bg-zinc-800 focus-visible:text-zinc-200
						{$playerState.silenceSkip ? 'text-indigo-400' : 'text-zinc-500'}"
					title="Skip silence (S)"
					aria-label="Toggle silence skip"
					aria-pressed={$playerState.silenceSkip}
				>
					{$playerState.silenceSkip ? 'SKIP' : 'skip'}
				</button>

				<!-- Speed -->
				<button
					onclick={() => {
						const speeds = [0.75, 1, 1.25, 1.5, 1.75, 2];
						const current = Math.round($playerState.playbackSpeed * 100);
						const idx = speeds.findIndex((s) => Math.round(s * 100) === current);
						const next = speeds[(idx + 1) % speeds.length];
						setSpeed(next);
					}}
					class="rounded px-1.5 py-0.5 font-mono tabular-nums transition-colors hover:bg-zinc-800 hover:text-zinc-200 focus-visible:text-zinc-200"
					title="Playback speed ([ / ])"
					aria-label="Playback speed"
				>
					{$playerState.playbackSpeed.toFixed(2)}x
				</button>

				<!-- Volume -->
				<div class="flex items-center gap-1">
					<button
						onclick={() => setVolume($playerState.volume > 0 ? 0 : 1)}
						class="rounded p-1 transition-colors hover:bg-zinc-800 hover:text-zinc-200 focus-visible:text-zinc-200"
						title="Mute/unmute"
						aria-label={$playerState.volume > 0 ? 'Mute' : 'Unmute'}
					>
						{#if $playerState.volume === 0}
							<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
								<path stroke-linecap="round" stroke-linejoin="round" d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"/>
								<path stroke-linecap="round" stroke-linejoin="round" d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2"/>
							</svg>
						{:else}
							<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
								<path stroke-linecap="round" stroke-linejoin="round" d="M15.536 8.464a5 5 0 010 7.072M18.364 5.636a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"/>
							</svg>
						{/if}
					</button>
					<input
						type="range"
						min="0"
						max="1"
						step="0.05"
						value={$playerState.volume}
						oninput={(e) => setVolume(Number((e.target as HTMLInputElement).value))}
						class="h-1 w-16 cursor-pointer appearance-none rounded-full bg-zinc-700 accent-indigo-500"
						aria-label="Volume"
					/>
				</div>
			</div>
		</div>
	</div>
{/if}
