outer: for (let i: number = 0; i < 3; i++) {
  for (let j: number = 0; j < 3; j++) {
    if (j === 1) {
      continue outer;
    }
    console.log(i, j);
  }
}
