import type { Client } from '@adrastos/lib';
import type { QueryClient } from '@tanstack/react-query';

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

export interface Upload {
  id: string;
  name: string;
  size: number;
  type: string;
  createdAt: Date;
}
