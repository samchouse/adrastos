import { Client } from '@adrastos/lib';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useAtomValue } from 'jotai';

import { clientAtom } from '~/lib/state';

import { queryKeys } from './queries';

export const useSignupMutation = () => {
  const client = useAtomValue(clientAtom);
  const { mutateAsync } = useLoginMutation(client);

  return useMutation({
    mutationKey: ['auth', 'signup'],
    mutationFn: async (
      data: Parameters<(typeof client)['accounts']['signup']>[0],
    ) => await client.accounts.signup(data),
    onSuccess: async (_, vars) => mutateAsync(vars),
  });
};

export const useLoginMutation = (client: Client) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['auth', 'login'],
    mutationFn: async (
      data: Parameters<(typeof client)['accounts']['login']>[0],
    ) => await client.accounts.login(data),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.tokenRefresh }),
  });
};

export const useLogoutMutation = (client: Client) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['auth', 'logout'],
    mutationFn: async () => await client.accounts.logout(),
    onSuccess: () => {
      client.authToken = undefined;
      void queryClient.resetQueries({ queryKey: queryKeys.tokenRefresh });
    },
  });
};

export const useConfigSmtpMutation = () => {
  const queryClient = useQueryClient();
  const client = useAtomValue(clientAtom);

  return useMutation({
    mutationKey: ['config', 'smtp'],
    mutationFn: async (
      data: Parameters<(typeof client)['config']['updateSmtp']>[0],
    ) => await client.config.updateSmtp(data),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.configDetails }),
  });
};

export const useConfigOAuth2Mutation = () => {
  const queryClient = useQueryClient();
  const client = useAtomValue(clientAtom);

  return useMutation({
    mutationKey: ['config', 'oauth2'],
    mutationFn: async (
      data: Parameters<(typeof client)['config']['updateOAuth2']>[0],
    ) => await client.config.updateOAuth2(data),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.configDetails }),
  });
};

export const useResendVerificationMutation = () => {
  const queryClient = useQueryClient();
  const client = useAtomValue(clientAtom);

  return useMutation({
    mutationKey: ['auth', 'resendVerification'],
    mutationFn: async () => await client.accounts.resendVerification(),
    onSuccess: () => queryClient.refetchQueries({ queryKey: queryKeys.me }),
  });
};

export const useCreateTableMutation = () => {
  const queryClient = useQueryClient();
  const client = useAtomValue(clientAtom);

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
