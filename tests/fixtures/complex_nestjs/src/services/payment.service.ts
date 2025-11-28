import { Injectable } from '@nestjs/common';
import { FraudService } from './fraud.service';
import { CreatePaymentDto } from '../dtos/payment.dto';

@Injectable()
export class PaymentService {
    // DEPENDENCY INJECTION HERE
    constructor(private fraudService: FraudService) { }

    async process(dto: CreatePaymentDto): Promise<string> {
        const isSafe = this.fraudService.check(dto.targetAccount);
        if (!isSafe) {
            return "BLOCKED";
        }
        return "PROCESSED_" + Math.round(dto.amount);
    }
}
