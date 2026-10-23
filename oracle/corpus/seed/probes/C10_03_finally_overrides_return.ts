function h(): number {
  try {
    return 1;
  } finally {
    return 2;
  }
}
console.log(h());
