import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { HealthModule } from './health/health.module';
import { ProductsModule } from './products/products.module';
import { ReportsModule } from './reports/reports.module';

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
    HealthModule,
    ProductsModule,
    ReportsModule,
  ],
})
export class AppModule {}
