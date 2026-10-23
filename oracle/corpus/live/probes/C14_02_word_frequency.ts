function countWords(text: string): Map<string, number> {
  const counts: Map<string, number> = new Map();
  const words: string[] = text.split(" ");
  for (const w of words) {
    const prev: number = counts.get(w) ?? 0;
    counts.set(w, prev + 1);
  }
  return counts;
}
const result: Map<string, number> = countWords("the cat the dog the end");
console.log(result.get("the"));
console.log(result.size);
