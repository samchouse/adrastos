import type { Client } from '../..';

export abstract class BaseModule {
  constructor(protected client: Client) {}
}
