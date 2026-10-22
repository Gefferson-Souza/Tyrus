export class DomainError extends Error {
  constructor(
    readonly code: string,
    readonly detail: string,
  ) {
    super(detail);
    this.name = 'DomainError';
  }
}
