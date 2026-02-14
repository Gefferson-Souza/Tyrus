enum Status {
  Active = "active",
  Inactive = "inactive"
}

// Async to allow throw tests if needed
async function runTest(arr: number[], status: Status): Promise<string> {
  let sum = 0;

  // for..of
  for (const item of arr) {
    // ternary
    const val = item > 5 ? 5 : item;

    // try-catch
    try {
      if (val < 0) throw "Negative";
      sum = sum + val;
    } catch (e) {
      console.log("Caught error");
    }

    // switch
    switch (val) {
      case 0:
        console.log("Zero");
        break;
      default:
        // do-while
        let j = 0;
        do { j = j + 1; } while (j < 1);
    }
  }

  if (status === Status.Active) {
    return "Active";
  }

  return "Done";
}
