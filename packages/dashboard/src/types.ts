import { Client } from '@adrastos/lib';
import { QueryClient } from '@tanstack/react-query';

export interface RouterContext {
  client: Client;
  queryClient: QueryClient;
}

export interface Team {
  id: string;
  name: string;
}

export interface Project {
  id: string;
  name: string;
  teamId: string;
  hostnames: string[];
}
