<script lang="ts">
	import { open } from '@tauri-apps/plugin-dialog';
	import { goto } from '$app/navigation';
	import { onMount, tick } from 'svelte';
	import { scanLibrary, assetUrl, getAllProgress } from '$lib/api/tauri';
	import { booksStore, ensureBooksLoaded, reloadBooks } from '$lib/stores/library';
	import { formatDuration, formatTime } from '$lib/utils/time';
	import { fuzzyFilter } from '$lib/utils/fuzzy';
	import type { Book, ScanResult, AllProgress } from '$lib/types/book';

	let scanning = $state(false);
	let scanResult = $state<ScanResult | null>(null);
	let error = $state('');
	let searchQuery = $state('');
	let gridEl = $state<HTMLElement>();
	let progressMap = $state<Map<number, AllProgress>>(new Map());

	async function loadProgress() {
		try {
			const all = await getAllProgress();
			progressMap = new Map(all.map((p) => [p.book_id, p]));
		} catch {}
	}
	let searchEl = $state<HTMLInputElement>();

	const books = $derived(
		fuzzyFilter($booksStore, searchQuery, (b) => `${b.title} ${b.author ?? ''}`)
	);

	onMount(() => {
		ensureBooksLoaded();
		loadProgress();
	});

	async function pickAndScan() {
		try {
			const selected = await open({ directory: true, title: 'Choose audiobook library folder' });
			if (!selected) return;
			scanning = true;
			error = '';
			scanResult = null;
			scanResult = await scanLibrary(selected);
			await reloadBooks();
			await loadProgress();
		} catch (e) {
			error = String(e);
		} finally {
			scanning = false;
		}
	}

	function getGridItems(): HTMLElement[] {
		return Array.from(gridEl?.querySelectorAll<HTMLElement>('[data-book-card]') ?? []);
	}

	function getGridColumns(): number {
		if (!gridEl) return 1;
		return getComputedStyle(gridEl).gridTemplateColumns.split(' ').length;
	}

	function focusGridItem(index: number) {
		const items = getGridItems();
		if (!items.length) return false;
		items[Math.max(0, Math.min(index, items.length - 1))].focus();
		return true;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.defaultPrevented) return;
		const target = e.target as HTMLElement;
		const inInput = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable;

		if (!inInput && (e.key === '/' || e.key === 'f') && !e.metaKey && !e.ctrlKey) {
			e.preventDefault();
			searchEl?.focus();
			return;
		}

		if (inInput && e.key === 'Escape') {
			e.preventDefault();
			(target as HTMLInputElement).blur();
			focusGridItem(0);
			return;
		}

		if (inInput && e.key === 'Enter') {
			e.preventDefault();
			focusGridItem(0);
			return;
		}

		if (inInput) return;

		const onCard = target.hasAttribute('data-book-card');
		const navKeys = ['ArrowRight', 'ArrowLeft', 'ArrowDown', 'ArrowUp', 'j', 'k', 'l', ';'];

		if (navKeys.includes(e.key)) {
			e.preventDefault();
			if (!onCard) {
				focusGridItem(0);
				return;
			}

			const items = getGridItems();
			const idx = items.indexOf(target);
			if (idx === -1) { focusGridItem(0); return; }

			const cols = getGridColumns();
			let next = idx;

			switch (e.key) {
				case 'ArrowRight': case ';': next = Math.min(idx + 1, items.length - 1); break;
				case 'ArrowLeft': case 'l': next = Math.max(idx - 1, 0); break;
				case 'ArrowDown': case 'j': next = Math.min(idx + cols, items.length - 1); break;
				case 'ArrowUp': case 'k': next = Math.max(idx - cols, 0); break;
			}

			if (next !== idx) items[next].focus();
			return;
		}

		if (e.key === 'a' && !e.metaKey && !e.ctrlKey) {
			e.preventDefault();
			pickAndScan();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="flex min-h-screen flex-col bg-zinc-900 text-zinc-100">
	<header class="flex items-center gap-4 border-b border-zinc-800 px-6 py-4">
		<div class="flex items-center gap-3 pr-2">
			<h1 class="text-xl font-bold tracking-tight">LitSync</h1>
			<img src="/favicon.png" alt="" class="h-8 w-8 rounded-lg" />
		</div>

		<div class="relative w-full max-w-2xl flex-1">
			<svg class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-zinc-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" aria-hidden="true">
				<path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
			</svg>
			<input
				bind:this={searchEl}
				bind:value={searchQuery}
				type="text"
				placeholder="Search books..."
				class="w-full rounded-lg border border-zinc-700 bg-zinc-800 py-2 pl-10 pr-10 text-sm text-zinc-100 placeholder-zinc-500 focus:border-indigo-500 focus:outline-none"
			/>
			{#if searchQuery}
				<button
					class="absolute right-2 top-1/2 -translate-y-1/2 rounded p-0.5 text-zinc-500 hover:text-zinc-300 focus-visible:text-zinc-300"
					onclick={() => { searchQuery = ''; searchEl?.focus(); }}
					aria-label="Clear search"
				>
					<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
						<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</button>
			{:else}
				<kbd class="absolute right-3 top-1/2 -translate-y-1/2 rounded bg-zinc-700 px-1.5 py-0.5 text-[10px] text-zinc-500">/</kbd>
			{/if}
		</div>

		<button
			onclick={pickAndScan}
			disabled={scanning}
			class="flex-shrink-0 rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-indigo-500 focus-visible:bg-indigo-500 disabled:opacity-50"
			title="Add library folder (A)"
		>
			{scanning ? 'Scanning...' : 'Add Library'}
		</button>
	</header>

	{#if scanResult}
		<div class="mx-6 mt-4 rounded-lg bg-zinc-800 p-3 text-sm" role="status">
			Found {scanResult.books_found} book{scanResult.books_found !== 1 ? 's' : ''},
			added {scanResult.books_added} new.
			{#if scanResult.errors.length > 0}
				<span class="text-amber-400">{scanResult.errors.length} error{scanResult.errors.length !== 1 ? 's' : ''}.</span>
			{/if}
		</div>
	{/if}

	{#if error}
		<div class="mx-6 mt-4 rounded-lg bg-red-900/50 p-3 text-sm text-red-200" role="alert">{error}</div>
	{/if}

	<main class="flex-1 p-6">
		{#if $booksStore.length === 0 && !scanning}
			<div class="flex flex-col items-center justify-center py-24 text-zinc-500">
				<svg class="mb-4 h-16 w-16" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
					<path stroke-linecap="round" stroke-linejoin="round" d="M12 6.042A8.967 8.967 0 006 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 016 18c2.305 0 4.408.867 6 2.292m0-14.25a8.966 8.966 0 016-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0018 18a8.967 8.967 0 00-6 2.292m0-14.25v14.25" />
				</svg>
				<p class="text-lg">No books yet</p>
				<p class="mt-1 text-sm">Press <kbd class="rounded bg-zinc-700 px-1.5 py-0.5 text-xs">A</kbd> to add a library folder</p>
			</div>
		{:else if books.length === 0 && searchQuery}
			<div class="flex flex-col items-center justify-center py-24 text-zinc-500">
				<p class="text-lg">No matches for "{searchQuery}"</p>
				<p class="mt-1 text-sm">Press <kbd class="rounded bg-zinc-700 px-1.5 py-0.5 text-xs">Esc</kbd> to clear</p>
			</div>
		{:else}
			<div
				bind:this={gridEl}
				class="grid grid-cols-2 gap-5 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
			>
				{#each books as book (book.id)}
					{@const prog = progressMap.get(book.id)}
					{@const pct = prog && book.total_duration_ms > 0 ? Math.min(100, Math.round((prog.position_ms / book.total_duration_ms) * 100)) : 0}
					<button
						data-book-card
						class="group cursor-pointer rounded-lg p-1 text-left transition-transform hover:scale-[1.02] hover:bg-zinc-800/50 focus-visible:bg-zinc-800/50"
						onclick={() => goto(`/book/${book.id}`)}
					>
						<div class="relative aspect-square overflow-hidden rounded-lg bg-zinc-800">
							{#if book.cover_path}
								<img
									src={assetUrl(book.cover_path)}
									alt=""
									class="h-full w-full object-cover"
									onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
								/>
							{:else}
								<div class="flex h-full w-full items-center justify-center bg-zinc-700 text-4xl font-bold text-zinc-500">
									{book.title.charAt(0)}
								</div>
							{/if}
							{#if prog?.status === 'finished'}
								<div class="absolute top-2 right-2 rounded bg-green-600/90 px-1.5 py-0.5 text-[10px] font-semibold text-white">Done</div>
							{:else if pct > 0}
								<!-- Progress bar at bottom of cover -->
								<div class="absolute bottom-0 left-0 right-0 h-1 bg-zinc-900/60">
									<div class="h-full bg-indigo-500 transition-all" style="width: {pct}%"></div>
								</div>
								<div class="absolute top-2 right-2 rounded bg-black/70 px-1.5 py-0.5 text-[10px] font-medium text-zinc-200">{pct}%</div>
							{/if}
						</div>
						<div class="mt-2 px-1">
							<p class="truncate text-sm font-medium text-zinc-100 group-hover:text-white group-focus-visible:text-white">{book.title}</p>
							{#if book.author}
								<p class="truncate text-xs text-zinc-400">{book.author}</p>
							{/if}
							<p class="mt-0.5 text-xs text-zinc-500">
								{#if prog && prog.position_ms > 0 && prog.status !== 'finished'}
									{formatTime(prog.position_ms)} / {formatDuration(book.total_duration_ms)}
								{:else}
									{formatDuration(book.total_duration_ms)} &middot; {book.file_count} file{book.file_count !== 1 ? 's' : ''}
								{/if}
							</p>
						</div>
					</button>
				{/each}
			</div>
		{/if}
	</main>
</div>
