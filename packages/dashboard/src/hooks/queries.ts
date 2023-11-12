import { useQuery } from '@tanstack/react-query';
import { useAtomValue } from 'jotai';

import { getTableData } from '~/lib';
import { clientAtom } from '~/lib/state';

export const queryKeys = {
  tokenRefresh: ['auth', 'token', 'refresh'] as const,
  me: ['me'] as const,
  configDetails: ['config', 'details'] as const,
  tables: ['tables'] as const,
  tableData: (table: string) => [...queryKeys.tables, table, 'data'] as const,
};

export const useTokenRefreshQuery = () => {
  const client = useAtomValue(clientAtom);

  return useQuery({
    queryKey: queryKeys.tokenRefresh,
    queryFn: async () => await client.accounts.refreshToken(),
    refetchInterval: 1000 * 60 * 10,
    retry: false,
  });
};

export const useMeQuery = () => {
  const client = useAtomValue(clientAtom);
  const { isSuccess } = useTokenRefreshQuery();

  return useQuery({
    queryKey: queryKeys.me,
    queryFn: async () => await client.accounts.currentUser(),
    enabled: isSuccess,
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
