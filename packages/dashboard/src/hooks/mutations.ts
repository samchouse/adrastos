'use client';

import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useAtomValue } from 'jotai';

import {
  client,
  deleteRow,
  getLogout,
  patchUpdateRow,
  postConfigOAuth2,
  postConfigSmtp,
  postCreateRow,
  postLogin,
  postResendVerification,
  postSignup,
} from '~/lib';
import { clientAtom } from '~/lib/state';

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
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.tokenRefresh }),
  });
};

export const useLogoutMutation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['auth', 'logout'],
    mutationFn: async () => await getLogout(),
    onSuccess: () => {
      client.defaults.headers.common.Authorization = undefined;
      void queryClient.resetQueries({ queryKey: queryKeys.tokenRefresh });
      void queryClient.resetQueries({ queryKey: queryKeys.me });
    },
  });
};

export const useConfigSmtpMutation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['config', 'smtp'],
    mutationFn: async (data: Parameters<typeof postConfigSmtp>[0]) =>
      await postConfigSmtp(data),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.configDetails }),
  });
};

export const useConfigOAuth2Mutation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['config', 'oauth2'],
    mutationFn: async (data: Parameters<typeof postConfigOAuth2>[0]) =>
      await postConfigOAuth2(data),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.configDetails }),
  });
};

export const useResendVerificationMutation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['auth', 'resendVerification'],
    mutationFn: async () => await postResendVerification(),
    onSuccess: () => queryClient.refetchQueries({ queryKey: queryKeys.me }),
  });
};

export const useCreateTableMutation = () => {
  const client = useAtomValue(clientAtom);
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['customTable', 'create'],
    mutationFn: async (
      table: Parameters<(typeof client)['tables']['create']>[0],
    ) => await client.tables.create(table),
    onSuccess: () => queryClient.refetchQueries({ queryKey: queryKeys.tables }),
  });
};

export const useCreateRowMutation = (table: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['customRow', 'create'],
    mutationFn: async (data: Parameters<typeof postCreateRow>[1]) =>
      await postCreateRow(table, data),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.tableData(table) }),
  });
};

export const useUpdateRowMutation = (table: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['customRow', 'update'],
    mutationFn: async ({
      id,
      data,
    }: {
      id: Parameters<typeof patchUpdateRow>[1];
      data: Parameters<typeof patchUpdateRow>[2];
    }) => await patchUpdateRow(table, id, data),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.tableData(table) }),
  });
};

export const useDeleteRowMutation = (table: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['customRow', 'delete'],
    mutationFn: async (id: Parameters<typeof deleteRow>[1]) =>
      await deleteRow(table, id),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.tableData(table) }),
  });
};
