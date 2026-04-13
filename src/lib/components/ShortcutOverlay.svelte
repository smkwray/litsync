<script lang="ts">
	let { open = $bindable(false) } = $props();

	const sections = [
		{
			title: 'Playback',
			shortcuts: [
				{ keys: ['Space'], desc: 'Play / Pause' },
				{ keys: ['←'], desc: 'Rewind 10s' },
				{ keys: ['→'], desc: 'Forward 10s' },
				{ keys: ['Shift', '←'], desc: 'Rewind 30s' },
				{ keys: ['Shift', '→'], desc: 'Forward 30s' },
				{ keys: ['['], desc: 'Speed down' },
				{ keys: [']'], desc: 'Speed up' },
				{ keys: ['S'], desc: 'Toggle silence skip' },
			],
		},
		{
			title: 'Navigation',
			shortcuts: [
				{ keys: ['N'], desc: 'Next chapter' },
				{ keys: ['P'], desc: 'Previous chapter' },
				{ keys: ['Esc'], desc: 'Back to library' },
				{ keys: ['↑', '↓', '←', '→'], desc: 'Move between items' },
				{ keys: ['J', 'K', 'L', ';'], desc: 'Down, Up, Left, Right' },
				{ keys: ['Enter'], desc: 'Open book / Play from last position' },
				{ keys: ['Tab'], desc: 'Next focusable element' },
			],
		},
		{
			title: 'Library',
			shortcuts: [
				{ keys: ['/'], desc: 'Focus search' },
				{ keys: ['Esc'], desc: 'Clear search / unfocus' },
				{ keys: ['A'], desc: 'Add library folder' },
			],
		},
		{
			title: 'General',
			shortcuts: [
				{ keys: ['?'], desc: 'Show this menu' },
				{ keys: ['⌘', ','], desc: 'Settings' },
			],
		},
	];

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape' || e.key === '?') {
			e.preventDefault();
			e.stopPropagation();
			open = false;
		}
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			open = false;
		}
	}

	let firstFocusable = $state<HTMLElement>();

	$effect(() => {
		if (open && firstFocusable) {
			firstFocusable.focus();
		}
	});
</script>

{#if open}
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div
		class="fixed inset-0 z-[100] flex items-center justify-center bg-black/60 backdrop-blur-sm"
		onkeydown={handleKeydown}
		onclick={handleBackdropClick}
		role="dialog"
		tabindex="-1"
		aria-label="Keyboard shortcuts"
		aria-modal="true"
	>
		<div class="mx-4 max-h-[80vh] w-full max-w-lg overflow-y-auto rounded-xl bg-zinc-800 shadow-2xl">
			<div class="flex items-center justify-between border-b border-zinc-700 px-6 py-4">
				<h2 class="text-lg font-semibold text-zinc-100">Keyboard Shortcuts</h2>
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

			<div class="space-y-6 px-6 py-5">
				{#each sections as section}
					<div>
						<h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-zinc-500">{section.title}</h3>
						<div class="space-y-1.5">
							{#each section.shortcuts as shortcut}
								<div class="flex items-center justify-between py-1">
									<span class="text-sm text-zinc-300">{shortcut.desc}</span>
									<div class="flex items-center gap-1">
										{#each shortcut.keys as key}
											<kbd class="inline-flex min-w-[1.75rem] items-center justify-center rounded bg-zinc-700 px-1.5 py-0.5 text-xs font-medium text-zinc-300">{key}</kbd>
										{/each}
									</div>
								</div>
							{/each}
						</div>
					</div>
				{/each}
			</div>
		</div>
	</div>
{/if}
