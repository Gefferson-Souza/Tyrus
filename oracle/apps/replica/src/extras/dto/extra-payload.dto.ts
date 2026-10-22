import { Transform, Type } from 'class-transformer';
import {
  ArrayMaxSize,
  IsArray,
  IsEmail,
  IsEnum,
  IsNotEmpty,
  IsString,
  ValidateNested,
} from 'class-validator';

export enum Priority {
  Low = 'low',
  High = 'high',
}

export class AddressDto {
  @IsString()
  @IsNotEmpty()
  city: string;

  @IsString()
  @IsNotEmpty()
  country: string;
}

export class ExtraPayloadDto {
  @Transform(({ value }: { value: unknown }) => (typeof value === 'string' ? value.trim() : value))
  @IsString()
  @IsNotEmpty()
  label: string;

  @IsEnum(Priority)
  priority: Priority;

  @IsEmail()
  contact: string;

  @IsArray()
  @ArrayMaxSize(3)
  @IsString({ each: true })
  tags: string[];

  @ValidateNested()
  @Type(() => AddressDto)
  address: AddressDto;
}
