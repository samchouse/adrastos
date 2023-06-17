import { useMutation, useQueryClient } from '@tanstack/react-query';

import { client, postConfigSmtp, postLogin } from '~/lib';

import { queryKeys } from './queries';

export const useLoginMutation = () =>
  useMutation({
    mutationKey: ['login'],
    mutationFn: async (data: Parameters<typeof postLogin>[0]) =>
      await postLogin(data),
    onSuccess: (data) => {
      client.defaults.headers.common[
        'Authorization'
      ] = `Bearer ${data.accessToken}`;
    }
  });

export const useConfigSmtpMutation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['config', 'smtp'],
    mutationFn: async (data: Parameters<typeof postConfigSmtp>[0]) =>
      await postConfigSmtp(data),
    onSuccess: () => queryClient.refetchQueries(queryKeys.configDetails)
  });
};
