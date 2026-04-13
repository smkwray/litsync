/**
 * Simple fuzzy match: checks if all characters in the query appear
 * in order in the target string. Returns a score (lower = better match)
 * or null if no match.
 */
export function fuzzyMatch(query: string, target: string): number | null {
	const q = query.toLowerCase();
	const t = target.toLowerCase();

	if (q.length === 0) return 0;
	if (q.length > t.length) return null;

	let qi = 0;
	let score = 0;
	let lastMatchIdx = -1;

	for (let ti = 0; ti < t.length && qi < q.length; ti++) {
		if (t[ti] === q[qi]) {
			// Bonus for consecutive matches
			const gap = ti - lastMatchIdx - 1;
			score += gap;
			// Bonus for matching at start of word
			if (ti === 0 || t[ti - 1] === ' ' || t[ti - 1] === '-' || t[ti - 1] === '_') {
				score -= 5;
			}
			lastMatchIdx = ti;
			qi++;
		}
	}

	if (qi < q.length) return null; // not all query chars matched
	return score;
}

/**
 * Filter and sort items by fuzzy match against a query.
 * Returns items that match, sorted by best match first.
 */
export function fuzzyFilter<T>(
	items: T[],
	query: string,
	getText: (item: T) => string,
): T[] {
	if (!query.trim()) return items;

	const scored: { item: T; score: number }[] = [];
	for (const item of items) {
		const text = getText(item);
		const score = fuzzyMatch(query, text);
		if (score !== null) {
			scored.push({ item, score });
		}
	}

	scored.sort((a, b) => a.score - b.score);
	return scored.map((s) => s.item);
}
