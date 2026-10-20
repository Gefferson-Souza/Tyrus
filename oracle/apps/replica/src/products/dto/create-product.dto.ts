import {
  IsBoolean,
  IsInt,
  IsNumber,
  IsOptional,
  IsString,
  Length,
  MaxLength,
  Matches,
  Min,
} from 'class-validator';

export class CreateProductDto {
  @IsString()
  @Length(3, 120)
  name: string;

  @IsString()
  @Matches(/^[A-Z0-9-]{4,32}$/)
  sku: string;

  @IsOptional()
  @IsString()
  @MaxLength(2000)
  description?: string;

  @IsNumber({ maxDecimalPlaces: 2 })
  @Min(0)
  price: number;

  @IsInt()
  @Min(0)
  stock: number;

  @IsInt()
  @Min(1)
  categoryId: number;

  @IsOptional()
  @IsBoolean()
  active?: boolean;
}
