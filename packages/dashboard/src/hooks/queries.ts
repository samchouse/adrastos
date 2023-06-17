import { useQuery } from '@tanstack/react-query';

import { getConfigDetails } from '~/lib';

export const queryKeys = {
  configDetails: ['configDetails'] as const
};

export const useConfigDetailsQuery = () =>
  useQuery({
    queryKey: queryKeys.configDetails,
    queryFn: async () => await getConfigDetails()
  });
