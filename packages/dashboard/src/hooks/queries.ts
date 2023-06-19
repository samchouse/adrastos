import { useQuery } from '@tanstack/react-query';

import { client, getConfigDetails, getTokenRefresh } from '~/lib';

export const queryKeys = {
  tokenRefresh: ['auth', 'token', 'refresh'] as const,
  configDetails: ['config', 'details'] as const
};

export const useTokenRefreshQuery = () =>
  useQuery({
    queryKey: queryKeys.tokenRefresh,
    queryFn: async () => await getTokenRefresh(),
    refetchInterval: 1000 * 60 * 10
  });

export const useConfigDetailsQuery = () =>
  useQuery({
    queryKey: queryKeys.configDetails,
    queryFn: async () => await getConfigDetails(),
    enabled: client.defaults.headers.Authorization !== undefined
  });
