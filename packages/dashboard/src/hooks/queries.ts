import { Client } from '@adrastos/lib';
import { queryOptions, useQuery } from '@tanstack/react-query';
import { useAtomValue } from 'jotai';

import { clientAtom, getTableData } from '~/lib';

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
    throwOnError: false,
    refetchInterval: 1000 * 60 * 10,
    retry: false,
  });

export const meQueryOptions = (client: Client) =>
  queryOptions({
    queryKey: queryKeys.me,
    queryFn: async () => await client.accounts.currentUser(),
  });

export const useTokenRefreshQuery = () => {
  const client = useAtomValue(clientAtom);

  return useQuery({
    queryKey: queryKeys.tokenRefresh,
    queryFn: async () => await client.accounts.refreshToken(),
    refetchInterval: 1000 * 60 * 10,
    retry: false,
  });
};

export const useConfigDetailsQuery = () => {
  const client = useAtomValue(clientAtom);
  const { isSuccess } = useTokenRefreshQuery();

  return useQuery({
    queryKey: queryKeys.configDetails,
    queryFn: async () => await client.config.details(),
    enabled: isSuccess,
  });
};

export const useTablesQuery = () => {
  const client = useAtomValue(clientAtom);
  const { isSuccess } = useTokenRefreshQuery();

  return useQuery({
    queryKey: queryKeys.tables,
    queryFn: async () => await client.tables.list(),
    enabled: isSuccess,
  });
};

export const useTableDataQuery = <T>(table: string) => {
  const { isSuccess } = useTokenRefreshQuery();

  return useQuery({
    queryKey: queryKeys.tableData(table),
    queryFn: async () => await getTableData<T>(table),
    enabled: isSuccess,
  });
};
