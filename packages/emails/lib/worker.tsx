import { render } from '@react-email/components';
import * as React from 'react';
import * as redis from 'redis';

import VerificationEmail from '../emails/verification';
import { env } from './env';

const client = redis.createClient({ url: env.DRAGONFLY_URL });

const worker = async () => {
  await client.connect();

  await client.subscribe('emails', async (token) => {
    await client.publish('html', render(<VerificationEmail token={token} />));
  });
};

void worker();
