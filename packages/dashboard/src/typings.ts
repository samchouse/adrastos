import { Client } from '@adrastos/lib';
import { QueryClient } from '@tanstack/react-query';

export interface RouterContext {
  client: Client;
  queryClient: QueryClient;
}
