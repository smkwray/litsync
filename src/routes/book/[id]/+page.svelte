<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { getBook, getProgress, assetUrl } from '$lib/api/tauri';
	import { formatDuration, formatTime } from '$lib/utils/time';
	import { playerState, loadAndPlay, playChapter, toggle } from '$lib/stores/player';
	import type { BookWithChapters, Progress } from '$lib/types/book';

	let { data } = $props();
	let book = $state<BookWithChapters | null>(null);
	let savedProgress = $state<Progress | null>(null);
	let error = $state('');
	let chapterListEl = $state<HTMLElement>();

	const isCurrentBook = $derived($playerState.bookId === book?.id);
	const savedPct = $derived(
		savedProgress && book && book.total_duration_ms > 0
			? Math.min(100, Math.round((savedProgress.position_ms / book.total_duration_ms) * 100))
			: 0
	);

	onMount(async () => {
		try {
			book = await getBook(data.id);
			savedProgress = await getProgress(data.id);
		} catch (e) {
			error = String(e);
		}
	});

	/** Play from last position (or beginning). loadAndPlay auto-restores saved progress. */
	async function startPlayback(fromChapter?: number) {
		if (!book) return;
		// If clicking a specific chapter, seek to its start
		const startMs = fromChapter !== undefined ? book.chapters[fromChapter]?.start_ms : undefined;
		await loadAndPlay(
			book.id,
			book.title,
			book.author,
			book.cover_path,
			book.chapters,
			startMs, // undefined = loadAndPlay will auto-restore from saved progress
		);
	}

	function onChapterClick(index: number) {
		if (!book) return;
		if (isCurrentBook) {
			playChapter(index);
		} else {
			startPlayback(index);
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.defaultPrevented) return;
		const target = e.target as HTMLElement;
		const inInput = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable;
		if (inInput) return;

		const onChapter = target.hasAttribute('data-chapter');

		// Enter: play/resume the book (or toggle if already playing this book)
		if (e.key === 'Enter' && !onChapter) {
			e.preventDefault();
			if (isCurrentBook) {
				toggle();
			} else {
				startPlayback(); // auto-resumes from last saved position
			}
			return;
		}

		// Chapter navigation: arrows + jk
		const isChapterNav = ['ArrowDown', 'ArrowUp', 'j', 'k'].includes(e.key);

		if (isChapterNav) {
			e.preventDefault();
			const items = Array.from(chapterListEl?.querySelectorAll<HTMLElement>('[data-chapter]') ?? []);
			if (!items.length) return;

			if (!onChapter) {
				items[0].focus();
				return;
			}

			const idx = items.indexOf(target);
			if (idx === -1) { items[0].focus(); return; }

			let next = idx;
			switch (e.key) {
				case 'ArrowDown': case 'j': next = Math.min(idx + 1, items.length - 1); break;
				case 'ArrowUp': case 'k': next = Math.max(idx - 1, 0); break;
			}
			if (next !== idx) items[next].focus();
			return;
		}

		if (e.key === 'Escape' || (e.key === 'Backspace' && !e.metaKey)) {
			e.preventDefault();
			goto('/');
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="min-h-screen bg-zinc-900 text-zinc-100">
	<header class="border-b border-zinc-800 px-6 py-4">
		<a
			href="/"
			class="inline-flex items-center gap-1 rounded px-2 py-1 text-sm text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-zinc-200 focus-visible:text-zinc-200"
		>
			<svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" aria-hidden="true">
				<path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
			</svg>
			Library
			<kbd class="ml-2 rounded bg-zinc-700 px-1 py-0.5 text-[10px] text-zinc-500">Esc</kbd>
		</a>
	</header>

	{#if error}
		<div class="mx-6 mt-6 rounded-lg bg-red-900/50 p-4 text-red-200" role="alert">{error}</div>
	{:else if book}
		<div class="mx-auto max-w-4xl p-6">
			<div class="flex gap-6">
				<div class="h-48 w-48 flex-shrink-0 overflow-hidden rounded-lg bg-zinc-800">
					{#if book.cover_path}
						<img src={assetUrl(book.cover_path)} alt={book.title} class="h-full w-full object-cover" />
					{:else}
						<div class="flex h-full w-full items-center justify-center bg-zinc-700 text-5xl font-bold text-zinc-500">
							{book.title.charAt(0)}
						</div>
					{/if}
				</div>

				<div class="flex flex-col justify-center">
					<h1 class="text-2xl font-bold">{book.title}</h1>
					{#if book.author}
						<p class="mt-1 text-lg text-zinc-300">{book.author}</p>
					{/if}
					{#if book.narrator}
						<p class="mt-0.5 text-sm text-zinc-400">Narrated by {book.narrator}</p>
					{/if}
					<p class="mt-2 text-sm text-zinc-500">
						{formatDuration(book.total_duration_ms)} &middot;
						{book.file_count} file{book.file_count !== 1 ? 's' : ''} &middot;
						{book.chapters.length} chapter{book.chapters.length !== 1 ? 's' : ''}
					</p>

					<!-- Saved progress -->
					{#if savedProgress && savedProgress.position_ms > 0}
						<div class="mt-3 space-y-1">
							<div class="flex items-center justify-between text-xs text-zinc-400">
								<span>
									{#if savedProgress.status === 'finished'}
										Finished
									{:else}
										{formatTime(savedProgress.position_ms)} / {formatDuration(book.total_duration_ms)} ({savedPct}%)
									{/if}
								</span>
								{#if savedProgress.playback_speed !== 1}
									<span class="font-mono">{savedProgress.playback_speed.toFixed(2)}x</span>
								{/if}
							</div>
							<div class="h-1.5 w-full overflow-hidden rounded-full bg-zinc-700">
								<div
									class="h-full rounded-full transition-all {savedProgress.status === 'finished' ? 'bg-green-500' : 'bg-indigo-500'}"
									style="width: {savedPct}%"
								></div>
							</div>
						</div>
					{/if}

					<div class="mt-4 flex items-center gap-3">
						<button
							class="inline-flex items-center gap-2 rounded-lg bg-indigo-600 px-5 py-2.5 text-sm font-medium text-white transition-colors hover:bg-indigo-500 focus-visible:bg-indigo-500"
							onclick={() => isCurrentBook ? toggle() : startPlayback()}
						>
							<svg class="h-4 w-4" fill="currentColor" viewBox="0 0 24 24">
								{#if isCurrentBook && $playerState.isPlaying}
									<path d="M6 4h4v16H6V4zm8 0h4v16h-4V4z"/>
								{:else}
									<path d="M8 5v14l11-7z"/>
								{/if}
							</svg>
							{#if isCurrentBook && $playerState.isPlaying}
								Pause
							{:else if savedProgress && savedProgress.position_ms > 0 && savedProgress.status !== 'finished'}
								Resume
							{:else}
								Play
							{/if}
						</button>
						<kbd class="rounded bg-zinc-700 px-1.5 py-0.5 text-[10px] text-zinc-500">Enter</kbd>
					</div>
				</div>
			</div>

			<div class="mt-8">
				<h2 class="mb-3 text-lg font-semibold">Chapters</h2>
				<div
					bind:this={chapterListEl}
					class="divide-y divide-zinc-800 rounded-lg bg-zinc-800/50"
				>
					{#each book.chapters as chapter, i (chapter.id)}
						{@const isActive = isCurrentBook && $playerState.currentChapterIndex === i}
						<button
							data-chapter
							class="flex w-full items-center justify-between px-4 py-3 text-left transition-colors
								{isActive ? 'bg-indigo-600/20 text-indigo-300' : 'hover:bg-zinc-700/50 focus-visible:bg-zinc-700/50'}"
							onclick={() => onChapterClick(i)}
						>
							<div class="flex items-center gap-3">
								{#if isActive && $playerState.isPlaying}
									<span class="flex w-8 items-center justify-end">
										<svg class="h-4 w-4 text-indigo-400" fill="currentColor" viewBox="0 0 24 24">
											<path d="M6 4h4v16H6V4zm8 0h4v16h-4V4z"/>
										</svg>
									</span>
								{:else if isActive}
									<span class="flex w-8 items-center justify-end">
										<svg class="h-4 w-4 text-indigo-400" fill="currentColor" viewBox="0 0 24 24">
											<path d="M8 5v14l11-7z"/>
										</svg>
									</span>
								{:else}
									<span class="w-8 text-right font-mono text-sm text-zinc-500">{i + 1}</span>
								{/if}
								<span class="text-sm">{chapter.title}</span>
							</div>
							<div class="flex items-center gap-3">
								<span class="text-xs text-zinc-500">{formatTime(chapter.start_ms)}</span>
								<span class="text-xs text-zinc-600">{formatDuration(chapter.end_ms - chapter.start_ms)}</span>
							</div>
						</button>
					{/each}
				</div>
			</div>
		</div>
	{:else}
		<div class="flex items-center justify-center py-24 text-zinc-500">Loading...</div>
	{/if}
</div>
