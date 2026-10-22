import { CanActivate, ExecutionContext, ForbiddenException, Injectable } from '@nestjs/common';

export const API_KEY_HEADER = 'x-api-key';
export const VALID_API_KEY = 'replica-key';

@Injectable()
export class ApiKeyGuard implements CanActivate {
  canActivate(context: ExecutionContext): boolean {
    const request = context.switchToHttp().getRequest<{ headers: Record<string, string | undefined> }>();
    if (request.headers[API_KEY_HEADER] !== VALID_API_KEY) {
      throw new ForbiddenException('Invalid API key');
    }
    return true;
  }
}
