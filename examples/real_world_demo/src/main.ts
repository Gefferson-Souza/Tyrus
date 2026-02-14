import { NestFactory } from "@nestjs/core";
import { AppModule } from "./app.module";

async function bootstrap() {
  const app = await NestFactory.create(AppModule);
  // Port 3001 to avoid conflicts
  await app.listen(3001);
}

bootstrap();
