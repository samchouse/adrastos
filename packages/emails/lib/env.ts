import { createEnv } from '@t3-oss/env-core';
import * as dotenv from 'dotenv';
import { z } from 'zod';

dotenv.config({ path: '../../.env' });

export const env = createEnv({
  clientPrefix: '',
  client: {},
  server: {
    CLIENT_URL: z.string().optional(),
    REDIS_URL: z.string(),
    REDIS_PREFIX: z.string().optional(),
  },
  runtimeEnv: process.env,
});
