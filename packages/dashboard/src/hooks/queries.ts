import { Client, Table } from '@adrastos/lib';
import { queryOptions, useQuery } from '@tanstack/react-query';
import { useAtomValue } from 'jotai';

import { clientAtom } from '~/lib';

export const queryKeys = {
  tokenRefresh: ['auth', 'token', 'refresh'] as const,
  me: ['me'] as const,
  configDetails: ['config', 'details'] as const,
  tables: ['tables'] as const,
  tableData: (table: string) => [...queryKeys.tables, table, 'data'] as const,
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

export const useConfigDetailsQuery = () => {
  const client = useAtomValue(clientAtom);

  return useQuery({
    queryKey: queryKeys.configDetails,
    queryFn: async () => await client.config.details(),
  });
};

export const useTablesQuery = () => {
  const client = useAtomValue(clientAtom);

  return useQuery({
    queryKey: queryKeys.tables,
    queryFn: async () => await client.tables.list(),
  });
};

export const useTableDataQuery = <T, U extends boolean>(
  table: T extends Table<infer _, infer U> ? U : string,
  one: U = true as U,
) => {
  const client = useAtomValue(clientAtom);

  return useQuery({
    queryKey: queryKeys.tableData(table),
    queryFn: async () => await client.tables.get<T, U>(table, one),
  });
};
