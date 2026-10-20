import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Between, Repository } from 'typeorm';
import { OrderItem } from '../orders/order-item.entity';

export interface CategorySales {
  categoryId: number;
  categoryName: string;
  items: number;
  units: number;
  revenue: number;
  avgTicket: number;
}

export interface TopProduct {
  productId: number;
  name: string;
  units: number;
  revenue: number;
}

export interface SalesByCategoryReport {
  from: string;
  to: string;
  totalItems: number;
  totalRevenue: number;
  categories: CategorySales[];
  topProducts: TopProduct[];
}

const TOP_PRODUCTS = 5;

@Injectable()
export class ReportsService {
  constructor(
    @InjectRepository(OrderItem)
    private readonly orderItems: Repository<OrderItem>,
  ) {}

  async salesByCategory(from: string, to: string): Promise<SalesByCategoryReport> {
    const rows = await this.orderItems.find({
      where: { soldAt: Between(new Date(from), new Date(to)) },
      relations: { product: { category: true } },
      order: { id: 'ASC' },
    });

    const categories = new Map<number, CategorySales>();
    const products = new Map<number, TopProduct>();
    let totalRevenue = 0;

    for (const row of rows) {
      const lineRevenue = row.quantity * parseFloat(row.unitPrice);
      const category = row.product.category;

      let categorySales = categories.get(category.id);
      if (!categorySales) {
        categorySales = {
          categoryId: category.id,
          categoryName: category.name,
          items: 0,
          units: 0,
          revenue: 0,
          avgTicket: 0,
        };
        categories.set(category.id, categorySales);
      }
      categorySales.items += 1;
      categorySales.units += row.quantity;
      categorySales.revenue += lineRevenue;

      let productSales = products.get(row.product.id);
      if (!productSales) {
        productSales = { productId: row.product.id, name: row.product.name, units: 0, revenue: 0 };
        products.set(row.product.id, productSales);
      }
      productSales.units += row.quantity;
      productSales.revenue += lineRevenue;

      totalRevenue += lineRevenue;
    }

    for (const categorySales of categories.values()) {
      categorySales.avgTicket = categorySales.revenue / categorySales.items;
    }

    // Total comparators: the output must not depend on the sort algorithm.
    const sortedCategories = [...categories.values()].sort(
      (a, b) => b.revenue - a.revenue || a.categoryId - b.categoryId,
    );
    const topProducts = [...products.values()]
      .sort((a, b) => b.revenue - a.revenue || a.productId - b.productId)
      .slice(0, TOP_PRODUCTS);

    return {
      from,
      to,
      totalItems: rows.length,
      totalRevenue,
      categories: sortedCategories,
      topProducts,
    };
  }
}
