import { useMutation, useQueryClient } from '@tanstack/react-query';

import {
  client,
  getLogout,
  postConfigOAuth2,
  postConfigSmtp,
  postLogin,
  postSignup
} from '~/lib';

import { queryKeys } from './queries';

export const useSignupMutation = () => {
  const { mutate } = useLoginMutation();

  return useMutation({
    mutationKey: ['auth', 'signup'],
    mutationFn: async (data: Parameters<typeof postSignup>[0]) =>
      await postSignup(data),
    onSuccess: (_, vars) => mutate(vars)
  });
};

export const useLoginMutation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['auth', 'login'],
    mutationFn: async (data: Parameters<typeof postLogin>[0]) =>
      await postLogin(data),
    onSuccess: () => queryClient.refetchQueries(queryKeys.tokenRefresh)
  });
};

export const useLogoutMutation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['auth', 'logout'],
    mutationFn: async () => await getLogout(),
    onSuccess: () => {
      client.defaults.headers.Authorization = '';
      queryClient.resetQueries(queryKeys.tokenRefresh);
      queryClient.resetQueries(queryKeys.me);
    }
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
