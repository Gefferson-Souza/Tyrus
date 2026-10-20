import { Injectable, NotFoundException } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { DeepPartial, Repository } from 'typeorm';
import { CreateProductDto } from './dto/create-product.dto';
import { Product } from './product.entity';

@Injectable()
export class ProductsService {
  constructor(
    @InjectRepository(Product)
    private readonly repo: Repository<Product>,
  ) {}

  async findOne(id: number): Promise<Product> {
    const product = await this.repo.findOne({ where: { id }, relations: { category: true } });
    if (!product) {
      throw new NotFoundException(`Product ${id} not found`);
    }
    return product;
  }

  async list(limit: number, offset: number): Promise<{ items: Product[]; limit: number; offset: number }> {
    const items = await this.repo.find({
      relations: { category: true },
      order: { id: 'ASC' },
      take: limit,
      skip: offset,
    });
    return { items, limit, offset };
  }

  // The request number is bound as-is and echoed back: `price` is a number in the POST response
  // even though reads return the numeric string.
  create(dto: CreateProductDto): Promise<Product> {
    return this.repo.save(this.repo.create(dto as unknown as DeepPartial<Product>));
  }
}
