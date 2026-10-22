import { Controller, Get } from '@nestjs/common';
import { SkipThrottle, Throttle } from '@nestjs/throttler';

@Controller('throttled')
export class ThrottledController {
  @Get('ping')
  ping(): { pong: boolean } {
    return { pong: true };
  }

  @Throttle({ default: { limit: 2, ttl: 60_000 } })
  @Get('strict')
  strict(): { strict: boolean } {
    return { strict: true };
  }

  @SkipThrottle()
  @Get('skipped')
  skipped(): { skipped: boolean } {
    return { skipped: true };
  }
}
