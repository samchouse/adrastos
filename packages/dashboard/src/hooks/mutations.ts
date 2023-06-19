import { useMutation, useQueryClient } from '@tanstack/react-query';

import { postConfigOAuth2, postConfigSmtp, postLogin, postSignup } from '~/lib';

import { queryKeys } from './queries';

export const useSignupMutation = () =>
  useMutation({
    mutationKey: ['auth', 'signup'],
    mutationFn: async (data: Parameters<typeof postSignup>[0]) =>
      await postSignup(data)
  });

export const useLoginMutation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['auth', 'login'],
    mutationFn: async (data: Parameters<typeof postLogin>[0]) =>
      await postLogin(data),
    onSuccess: () => queryClient.refetchQueries(queryKeys.tokenRefresh)
  });
};

export const useConfigSmtpMutation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['config', 'smtp'],
    mutationFn: async (data: Parameters<typeof postConfigSmtp>[0]) =>
      await postConfigSmtp(data),
    onSuccess: () => queryClient.refetchQueries(queryKeys.configDetails)
  });
};

export const useConfigOAuth2Mutation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['config', 'oauth2'],
    mutationFn: async (data: Parameters<typeof postConfigOAuth2>[0]) =>
      await postConfigOAuth2(data),
    onSuccess: () => queryClient.refetchQueries(queryKeys.configDetails)
  });
};
