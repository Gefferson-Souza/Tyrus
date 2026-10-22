import { Module } from '@nestjs/common';
import { APP_GUARD } from '@nestjs/core';
import { ThrottlerGuard, ThrottlerModule } from '@nestjs/throttler';
import { TypeOrmModule } from '@nestjs/typeorm';
import { ExtrasModule } from './extras/extras.module';
import { HealthModule } from './health/health.module';
import { ProductsModule } from './products/products.module';
import { ReportsModule } from './reports/reports.module';
import { ThrottledController } from './throttled/throttled.controller';
import { ThrottledModule } from './throttled/throttled.module';

const THROTTLE_TTL_MS = 60_000;
const THROTTLE_LIMIT = 5;

@Module({
  imports: [
    TypeOrmModule.forRoot({
      type: 'postgres',
      url: process.env.DATABASE_URL,
      autoLoadEntities: true,
      synchronize: false,
      // SQL logging is switched on only for the wire-capture runs.
      logging: process.env.TYPEORM_LOGGING === '1',
    }),
    ThrottlerModule.forRoot({
      throttlers: [{ ttl: THROTTLE_TTL_MS, limit: THROTTLE_LIMIT }],
      skipIf: (context) => context.getClass() !== ThrottledController,
    }),
    HealthModule,
    ProductsModule,
    ReportsModule,
    ThrottledModule,
    ExtrasModule,
  ],
  providers: [{ provide: APP_GUARD, useClass: ThrottlerGuard }],
})
export class AppModule {}
