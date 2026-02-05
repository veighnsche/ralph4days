/**
 * Generate a 4-letter acronym from a name
 * Mirrors the backend logic in crates/yaml-db/src/acronym.rs
 */
export function generateAcronym(name: string, displayName?: string): string {
  const source = displayName?.includes(" ") ? displayName : name;
  const cleaned = source.toUpperCase().replace(/-/g, " ").replace(/_/g, " ");
  const words = cleaned.split(/\s+/).filter((w) => w.length > 0);

  if (words.length === 0) return "UNKN";

  if (words.length === 1) {
    const word = words[0];
    if (word.length >= 4) {
      // Extract consonants preferentially, fall back to first 4
      const consonants = word
        .split("")
        .filter((c) => !"AEIOU".includes(c))
        .slice(0, 4)
        .join("");

      if (consonants.length >= 4) {
        return consonants;
      }
      return word.slice(0, 4);
    }
    // Pad short words by repeating last letter
    const lastChar = word[word.length - 1];
    return (word + lastChar.repeat(4 - word.length)).toUpperCase();
  }

  if (words.length === 2) {
    // 2 letters from each word
    return (words[0].slice(0, 2) + words[1].slice(0, 2)).toUpperCase();
  }

  if (words.length === 3) {
    // 2 from first, 1 from second, 1 from third
    return (words[0].slice(0, 2) + words[1][0] + words[2][0]).toUpperCase();
  }

  // 4+ words: first letter from first 4 words
  return words
    .slice(0, 4)
    .map((w) => w[0])
    .join("")
    .toUpperCase();
}

/**
 * Normalize feature name to lowercase with hyphens
 */
export function normalizeFeatureName(name: string): string {
  return name.toLowerCase().trim().replace(/\s+/g, "-").replace(/_+/g, "-");
}
