import { Client, Table } from '@adrastos/lib';
import { queryOptions } from '@tanstack/react-query';

const baseQueryKey = {
  tables: ['tables'] as const,
};

export const queryKeys = {
  tokenRefresh: ['auth', 'token', 'refresh'] as const,
  me: ['me'] as const,
  passkeys: ['passkeys', 'list'] as const,
  configDetails: ['config', 'details'] as const,
  tables: [...baseQueryKey.tables, 'list'] as const,
  tableData: (table: string) =>
    [...baseQueryKey.tables, table, 'data'] as const,
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
    queryFn: async () => await client.config.details(),
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
