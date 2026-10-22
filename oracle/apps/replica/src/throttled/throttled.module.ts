import { Module } from '@nestjs/common';
import { ThrottledController } from './throttled.controller';

@Module({ controllers: [ThrottledController] })
export class ThrottledModule {}
