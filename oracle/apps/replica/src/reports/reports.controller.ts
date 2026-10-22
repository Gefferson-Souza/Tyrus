import { Controller, Get, Query } from '@nestjs/common';
import { SalesReportQuery } from './dto/sales-report.query';
import { ReportsService, SalesByCategoryReport } from './reports.service';

@Controller('reports')
export class ReportsController {
  constructor(private readonly reports: ReportsService) {}

  @Get('sales-by-category')
  salesByCategory(@Query() query: SalesReportQuery): Promise<SalesByCategoryReport> {
    return this.reports.salesByCategory(query.from, query.to);
  }
}
