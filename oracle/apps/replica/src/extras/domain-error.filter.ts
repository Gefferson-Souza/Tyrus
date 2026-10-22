import { ArgumentsHost, Catch, ExceptionFilter } from '@nestjs/common';
import { HttpAdapterHost } from '@nestjs/core';
import { DomainError } from './domain-error';

const UNPROCESSABLE_ENTITY = 422;

@Catch(DomainError)
export class DomainErrorFilter implements ExceptionFilter<DomainError> {
  constructor(private readonly adapterHost: HttpAdapterHost) {}

  catch(exception: DomainError, host: ArgumentsHost): void {
    const response = host.switchToHttp().getResponse<unknown>();
    this.adapterHost.httpAdapter.reply(
      response,
      { code: exception.code, detail: exception.detail },
      UNPROCESSABLE_ENTITY,
    );
  }
}
