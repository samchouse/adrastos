import type { Client, Table } from '@adrastos/lib';
import { queryOptions } from '@tanstack/react-query';

import type { Project, Team, Upload } from '~/types';

const baseQueryKey = {
  tables: ['tables'] as const,
  teams: ['teams'] as const,
  storage: ['storage'] as const,
};

export const queryKeys = {
  tokenRefresh: ['auth', 'token', 'refresh'] as const,
  me: ['me'] as const,
  passkeys: ['passkeys', 'list'] as const,
  configDetails: ['config', 'details'] as const,
  tables: [...baseQueryKey.tables, 'list'] as const,
  tableData: (table: string) =>
    [...baseQueryKey.tables, table, 'data'] as const,
  teams: [...baseQueryKey.teams, 'list'] as const,
  projects: (teamId: string) =>
    [...baseQueryKey.teams, teamId, 'projects', 'list'] as const,
  project: (projectId: string) => ['projects', projectId] as const,
  storage: [...baseQueryKey.storage, 'list'] as const,
};

export function tokenRefreshQueryOptions(client: Client) {
  return queryOptions({
    queryKey: queryKeys.tokenRefresh,
    queryFn: async () => await client.accounts.refreshToken(),
    refetchInterval: 1000 * 60 * 10,
    retry: false,
  });
}

export function meQueryOptions(client: Client) {
  return queryOptions({
    queryKey: queryKeys.me,
    queryFn: () => client.accounts.currentUser(),
  });
}

export function configDetailsQueryOptions(client: Client) {
  return queryOptions({
    queryKey: queryKeys.configDetails,
    queryFn: async () =>
      await client.json<ReturnType<(typeof client)['config']['details']>>({
        path: '/config/details',
        method: 'GET',
        projectIdNeeded: true,
      }),
  });
}

export function tablesQueryOptions(client: Client) {
  return queryOptions({
    queryKey: queryKeys.tables,
    queryFn: () => client.tables.list(),
  });
}

export function tableDataQueryOptions<T, U extends boolean>(
  client: Client,
  table: T extends Table<infer _, infer U> ? U : string,
  one: U = true as U,
) {
  return queryOptions({
    queryKey: queryKeys.tableData(table),
    queryFn: () => client.tables.get<T, U>(table, one),
  });
}

export function passkeysQueryOptions(client: Client) {
  return queryOptions({
    queryKey: queryKeys.passkeys,
    queryFn: () => client.accounts.listPasskeys(),
  });
}

export function teamsQueryOptions(client: Client) {
  return queryOptions({
    queryKey: queryKeys.teams,
    queryFn: async () =>
      await client.json<Team[]>({
        method: 'GET',
        path: '/teams/list',
      }),
  });
}

export function projectsQueryOptions(client: Client, teamId: string) {
  return queryOptions({
    queryKey: queryKeys.projects(teamId),
    queryFn: async () =>
      await client.json<Project[]>({
        method: 'GET',
        path: `/teams/${teamId}/projects/list`,
      }),
  });
}

export function projectQueryOptions(client: Client, projectId: string) {
  return queryOptions({
    queryKey: queryKeys.project(projectId),
    queryFn: async () =>
      await client.json<Project>({
        method: 'GET',
        path: `/teams/projects/${projectId}`,
      }),
  });
}

export function storageQueryOptions(client: Client) {
  return queryOptions({
    queryKey: queryKeys.storage,
    queryFn: async () =>
      await client.json<Upload[]>({
        method: 'GET',
        path: '/storage/list',
        projectIdNeeded: true,
      }),
  });
}
