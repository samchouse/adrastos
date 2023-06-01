import { createEnv } from '@t3-oss/env-core';
import * as dotenv from 'dotenv';
import { z } from 'zod';

dotenv.config({ path: '../../.env' });

export const env = createEnv({
  clientPrefix: '',
  client: {},
  server: {
    SERVER_URL: z.string().optional(),
    DRAGONFLY_URL: z.string()
  },
  runtimeEnv: process.env
});
