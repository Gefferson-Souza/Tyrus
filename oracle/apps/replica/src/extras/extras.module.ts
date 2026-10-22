import { Module } from '@nestjs/common';
import { ExtrasController } from './extras.controller';

@Module({ controllers: [ExtrasController] })
export class ExtrasModule {}
