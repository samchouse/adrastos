import { Client, Table } from '@adrastos/lib';
import { queryOptions } from '@tanstack/react-query';

import { Project, Team } from '~/types';

const baseQueryKey = {
  tables: ['tables'] as const,
  teams: ['teams'] as const,
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
};

export const tokenRefreshQueryOptions = (client: Client) =>
  queryOptions({
    queryKey: queryKeys.tokenRefresh,
    queryFn: async () => await client.accounts.refreshToken(),
    refetchInterval: 1000 * 60 * 10,
    retry: false,
  });

export const meQueryOptions = (client: Client) =>
  queryOptions({
    queryKey: queryKeys.me,
    queryFn: () => client.accounts.currentUser(),
  });

export const configDetailsQueryOptions = (client: Client) =>
  queryOptions({
    queryKey: queryKeys.configDetails,
    queryFn: async () =>
      await client.request<ReturnType<(typeof client)['config']['details']>>({
        path: '/config/details',
        method: 'GET',
        projectIdNeeded: true,
      }),
  });

export const tablesQueryOptions = (client: Client) =>
  queryOptions({
    queryKey: queryKeys.tables,
    queryFn: () => client.tables.list(),
  });

export const tableDataQueryOptions = <T, U extends boolean>(
  client: Client,
  table: T extends Table<infer _, infer U> ? U : string,
  one: U = true as U,
) =>
  queryOptions({
    queryKey: queryKeys.tableData(table),
    queryFn: () => client.tables.get<T, U>(table, one),
  });

export const passkeysQueryOptions = (client: Client) =>
  queryOptions({
    queryKey: queryKeys.passkeys,
    queryFn: () => client.accounts.listPasskeys(),
  });

export const teamsQueryOptions = (client: Client) =>
  queryOptions({
    queryKey: queryKeys.teams,
    queryFn: async () =>
      await client.request<Team[]>({
        method: 'GET',
        path: '/teams/list',
      }),
  });

export const projectsQueryOptions = (client: Client, teamId: string) =>
  queryOptions({
    queryKey: queryKeys.projects(teamId),
    queryFn: async () =>
      await client.request<Project[]>({
        method: 'GET',
        path: `/teams/${teamId}/projects/list`,
      }),
  });

export const projectQueryOptions = (client: Client, projectId: string) =>
  queryOptions({
    queryKey: queryKeys.project(projectId),
    queryFn: async () =>
      await client.request<Project>({
        method: 'GET',
        path: `/teams/projects/${projectId}`,
      }),
  });
