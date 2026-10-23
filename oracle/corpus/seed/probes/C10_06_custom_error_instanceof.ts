class AppError extends Error {
  code: number;
  constructor(message: string, code: number) {
    super(message);
    this.code = code;
  }
}
try {
  throw new AppError("bad", 7);
} catch (e) {
  if (e instanceof AppError) {
    console.log(e.message + ":" + e.code);
  }
}
