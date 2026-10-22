import { CallHandler, ExecutionContext, Injectable, NestInterceptor } from '@nestjs/common';
import { map, Observable } from 'rxjs';

export interface Envelope<T> {
  data: T;
  meta: { handledBy: string };
}

@Injectable()
export class EnvelopeInterceptor<T> implements NestInterceptor<T, Envelope<T>> {
  intercept(_context: ExecutionContext, next: CallHandler<T>): Observable<Envelope<T>> {
    return next.handle().pipe(map((data) => ({ data, meta: { handledBy: 'envelope' } })));
  }
}
