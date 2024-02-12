import { render } from '@react-email/components';
import { pino } from 'pino';
import * as React from 'react';
import * as redis from 'redis';

import VerificationEmail from '../emails/verification.js';
import { env } from './env.js';

const logger = pino();
const client = redis.createClient({ url: env.REDIS_URL });

logger.info('Worker started');

const worker = async () => {
  await client.connect();

  await client.subscribe('emails', async (token) => {
    logger.info(`Received request with token: ${token}`);
    await client.publish(
      'html',
      render(
        <VerificationEmail
          token={token}
          baseUrl={`${env.CLIENT_URL ?? 'http://localhost:3000'}/api`}
        />,
      ),
    );
    logger.info(`Finished request with token: ${token}`);
  });
};

void worker();
