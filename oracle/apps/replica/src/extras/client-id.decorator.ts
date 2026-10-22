import { createParamDecorator, ExecutionContext } from '@nestjs/common';

export const CLIENT_ID_HEADER = 'x-client-id';
export const ANONYMOUS_CLIENT = 'anonymous';

export const ClientId = createParamDecorator((_data: unknown, context: ExecutionContext): string => {
  const request = context.switchToHttp().getRequest<{ headers: Record<string, string | undefined> }>();
  return request.headers[CLIENT_ID_HEADER] ?? ANONYMOUS_CLIENT;
});
