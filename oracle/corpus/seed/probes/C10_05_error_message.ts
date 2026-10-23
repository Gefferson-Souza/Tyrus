try {
  throw new Error("boom");
} catch (e) {
  console.log((e as Error).message);
}
