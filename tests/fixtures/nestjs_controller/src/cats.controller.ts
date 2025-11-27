import { Body, Controller, Get, Post } from '@nestjs/common';

export class CreateCatDto {
    name: string;
    age: number;
}

@Controller('cats')
export class CatsController {
    @Get()
    async findAll(): Promise<string> {
        return "This action returns all cats";
    }

    @Post()
    async create(@Body() createCatDto: CreateCatDto): Promise<CreateCatDto> {
        return createCatDto;
    }
}
