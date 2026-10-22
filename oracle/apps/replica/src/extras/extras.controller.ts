import {
  Body,
  Controller,
  Get,
  Param,
  ParseIntPipe,
  Post,
  UseFilters,
  UseGuards,
  UseInterceptors,
} from '@nestjs/common';
import { ApiKeyGuard } from './api-key.guard';
import { ClientId } from './client-id.decorator';
import { DomainError } from './domain-error';
import { DomainErrorFilter } from './domain-error.filter';
import { ExtraPayloadDto } from './dto/extra-payload.dto';
import { EnvelopeInterceptor } from './envelope.interceptor';

@Controller('extras')
@UseGuards(ApiKeyGuard)
@UseInterceptors(EnvelopeInterceptor)
@UseFilters(DomainErrorFilter)
export class ExtrasController {
  @Get('whoami')
  whoami(@ClientId() clientId: string): { clientId: string } {
    return { clientId };
  }

  @Get('items/:id')
  item(@Param('id', ParseIntPipe) id: number): { id: number; doubled: number } {
    return { id, doubled: id * 2 };
  }

  @Get('domain-error')
  domainError(): never {
    throw new DomainError('E_DEMO', 'Domain failure used by the replica');
  }

  @Post('validate')
  validate(@Body() payload: ExtraPayloadDto): ExtraPayloadDto {
    return payload;
  }
}
