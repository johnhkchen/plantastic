/** Centralized color-map objects for badge/tag/status styling across components.
 *  All class strings reference design-token utilities defined in tokens.css. */

/** Project status badge colors (draft, quoted, approved, complete). */
export const STATUS_COLORS: Record<string, string> = {
	draft: 'bg-surface-hover text-text-secondary',
	quoted: 'bg-info-surface text-info',
	approved: 'bg-success-surface text-success-text',
	complete: 'bg-purple-100 text-purple-700'
};

/** Material category filter colors (active/inactive pill states). */
export const CATEGORY_COLORS: Record<string, { active: string; inactive: string }> = {
	hardscape: {
		active: 'bg-stone-100 text-stone-800 border-stone-300',
		inactive: 'text-stone-600 hover:bg-stone-50'
	},
	softscape: {
		active: 'bg-green-100 text-green-800 border-green-300',
		inactive: 'text-green-600 hover:bg-green-50'
	},
	edging: {
		active: 'bg-amber-100 text-amber-800 border-amber-300',
		inactive: 'text-amber-600 hover:bg-amber-50'
	},
	fill: {
		active: 'bg-orange-100 text-orange-800 border-orange-300',
		inactive: 'text-orange-600 hover:bg-orange-50'
	}
};

/** Category heading text colors for material picker section headers. */
export const CATEGORY_HEADER_COLORS: Record<string, string> = {
	hardscape: 'text-stone-600',
	softscape: 'text-green-600',
	edging: 'text-amber-600',
	fill: 'text-orange-600'
};

/** Zone type badge colors for zone lists. */
export const ZONE_BADGE_COLORS: Record<string, string> = {
	bed: 'bg-amber-800 text-white',
	patio: 'bg-stone-500 text-white',
	path: 'bg-yellow-600 text-white',
	lawn: 'bg-green-600 text-white',
	wall: 'bg-gray-600 text-white',
	edging: 'bg-amber-600 text-white'
};

/** Quote tier display config (label + token-based color classes). */
export const TIER_CONFIG: Record<string, { label: string; color: string; bg: string }> = {
	good: { label: 'Good', color: 'text-tier-good', bg: 'bg-tier-good-surface' },
	better: { label: 'Better', color: 'text-tier-better', bg: 'bg-tier-better-surface' },
	best: { label: 'Best', color: 'text-tier-best', bg: 'bg-tier-best-surface' }
};

/** Material category badge colors for catalog cards. */
export const CATEGORY_BADGE_COLORS: Record<string, string> = {
	hardscape: 'bg-stone-100 text-stone-700',
	softscape: 'bg-green-100 text-green-700',
	edging: 'bg-amber-100 text-amber-700',
	fill: 'bg-orange-100 text-orange-700'
};
