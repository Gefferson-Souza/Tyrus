import { Controller, Post, Body, Get } from '@nestjs/common';
import { PaymentService } from '../services/payment.service';
import { CreatePaymentDto } from '../dtos/payment.dto';

@Controller('payments')
export class PaymentController {
    // DEPENDENCY INJECTION HERE
    constructor(private paymentService: PaymentService) { }

    @Post()
    async create(@Body() dto: CreatePaymentDto): Promise<string> {
        return await this.paymentService.process(dto);
    }

    @Get('/health')
    health(): string {
        return "OK";
    }
}
