import { Client } from '../..';

export abstract class BaseModule {
  // eslint-disable-next-line no-unused-vars
  constructor(protected client: Client) {}
}
