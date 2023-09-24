import { useMutation, useQueryClient } from '@tanstack/react-query';

import {
  client,
  deleteRow,
  getLogout,
  postConfigOAuth2,
  postConfigSmtp,
  postCreateRow,
  postCreateTable,
  postLogin,
  postResendVerification,
  postSignup,
} from '~/lib';

import { queryKeys } from './queries';

export const useSignupMutation = () => {
  const { mutateAsync } = useLoginMutation();

  return useMutation({
    mutationKey: ['auth', 'signup'],
    mutationFn: async (data: Parameters<typeof postSignup>[0]) =>
      await postSignup(data),
    onSuccess: async (_, vars) => mutateAsync(vars),
  });
};

export const useLoginMutation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['auth', 'login'],
    mutationFn: async (data: Parameters<typeof postLogin>[0]) =>
      await postLogin(data),
    onSuccess: () => queryClient.refetchQueries(queryKeys.tokenRefresh),
  });
};

export const useLogoutMutation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['auth', 'logout'],
    mutationFn: async () => await getLogout(),
    onSuccess: () => {
      client.defaults.headers.common.Authorization = undefined;
      queryClient.resetQueries(queryKeys.tokenRefresh);
      queryClient.resetQueries(queryKeys.me);
    },
  });
};

export const useConfigSmtpMutation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['config', 'smtp'],
    mutationFn: async (data: Parameters<typeof postConfigSmtp>[0]) =>
      await postConfigSmtp(data),
    onSuccess: () => queryClient.refetchQueries(queryKeys.configDetails),
  });
};

export const useConfigOAuth2Mutation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['config', 'oauth2'],
    mutationFn: async (data: Parameters<typeof postConfigOAuth2>[0]) =>
      await postConfigOAuth2(data),
    onSuccess: () => queryClient.refetchQueries(queryKeys.configDetails),
  });
};

export const useResendVerificationMutation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['auth', 'resendVerification'],
    mutationFn: async () => await postResendVerification(),
    onSuccess: () => queryClient.refetchQueries(queryKeys.me),
  });
};

export const useCreateTableMutation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['customTable', 'create'],
    mutationFn: async (data: Parameters<typeof postCreateTable>[0]) =>
      await postCreateTable(data),
    onSuccess: () => queryClient.refetchQueries(queryKeys.tables),
  });
};

export const useCreateRowMutation = (table: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['customRow', 'create'],
    mutationFn: async (data: Parameters<typeof postCreateRow>[1]) =>
      await postCreateRow(table, data),
    onSuccess: () => queryClient.refetchQueries(queryKeys.tableData(table)),
  });
};

export const useDeleteRowMutation = (table: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['customRow', 'delete'],
    mutationFn: async (id: Parameters<typeof deleteRow>[1]) =>
      await deleteRow(table, id),
    onSuccess: () => queryClient.refetchQueries(queryKeys.tableData(table)),
  });
};
