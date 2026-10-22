import 'reflect-metadata';
import { INestApplication } from '@nestjs/common';
import { NestFactory } from '@nestjs/core';
import { AppModule } from './app.module';
import { configureApp } from './app.setup';

const ADAPTERS = ['express', 'fastify'] as const;
type Adapter = (typeof ADAPTERS)[number];

function readAdapter(): Adapter {
  const raw = process.env.ADAPTER ?? 'express';
  const adapter = ADAPTERS.find((candidate) => candidate === raw);
  if (adapter === undefined) {
    throw new Error(`ADAPTER must be one of ${ADAPTERS.join('|')}, received "${raw}"`);
  }
  return adapter;
}

async function createApp(adapter: Adapter): Promise<INestApplication> {
  const options = { logger: ['error', 'warn'] as ('error' | 'warn')[] };
  if (adapter === 'fastify') {
    const { FastifyAdapter } = await import('@nestjs/platform-fastify');
    return NestFactory.create(AppModule, new FastifyAdapter(), options);
  }
  return NestFactory.create(AppModule, options);
}

async function bootstrap(): Promise<void> {
  const app = await createApp(readAdapter());
  configureApp(app);
  await app.listen(Number(process.env.PORT ?? 3000), process.env.LISTEN_HOST ?? '127.0.0.1');
}

void bootstrap();
