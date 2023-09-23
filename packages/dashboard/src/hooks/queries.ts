import { useQuery } from '@tanstack/react-query';

import {
  getConfigDetails,
  getMe,
  getTableData,
  getTables,
  getTokenRefresh,
} from '~/lib';

export const queryKeys = {
  tokenRefresh: ['auth', 'token', 'refresh'] as const,
  me: ['me'] as const,
  configDetails: ['config', 'details'] as const,
  tables: ['tables'] as const,
  createTable: () => [...queryKeys.tables, 'create'] as const,
};

export const useTokenRefreshQuery = () =>
  useQuery({
    queryKey: queryKeys.tokenRefresh,
    queryFn: async () => await getTokenRefresh(),
    refetchInterval: 1000 * 60 * 10,
    retry: false,
  });

export const useMeQuery = () => {
  const { isSuccess } = useTokenRefreshQuery();

  return useQuery({
    queryKey: queryKeys.me,
    queryFn: async () => await getMe(),
    enabled: isSuccess,
  });
};

export const useConfigDetailsQuery = () => {
  const { isSuccess } = useTokenRefreshQuery();

  return useQuery({
    queryKey: queryKeys.configDetails,
    queryFn: async () => await getConfigDetails(),
    enabled: isSuccess,
  });
};

export const useTablesQuery = () => {
  const { isSuccess } = useTokenRefreshQuery();

  return useQuery({
    queryKey: queryKeys.tables,
    queryFn: async () => await getTables(),
    enabled: isSuccess,
  });
};

export const useTableDataQuery = <T>(table: string) => {
  const { isSuccess } = useTokenRefreshQuery();

  return useQuery({
    queryKey: ['data'],
    queryFn: async () => await getTableData<T>(table),
    enabled: isSuccess,
  });
};
