@Injectable()
class CatsService {
  findAll(): string {
    return 'This action returns all cats';
  }
}

@Controller('cats')
class CatsController {
  constructor(private catsService: CatsService) { }

  @Get()
  findAll(): string {
    return this.catsService.findAll();
  }
}

@Module({
  controllers: [CatsController],
  providers: [CatsService],
})
class CatsModule { }
