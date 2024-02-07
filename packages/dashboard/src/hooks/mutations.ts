import { Client } from '@adrastos/lib';
import { useMutation, useQueryClient } from '@tanstack/react-query';

import { queryKeys } from './queries';

export const useSignupMutation = (client: Client) => {
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

export const useLogoutMutation = (client: Client) =>
  useMutation({
    mutationKey: ['auth', 'logout'],
    mutationFn: async () => await client.accounts.logout(),
    onSuccess: () => {
      client.authToken = undefined;
    },
  });

export const useConfigSmtpMutation = (client: Client) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['config', 'smtp'],
    mutationFn: async (
      data: Parameters<(typeof client)['config']['updateSmtp']>[0],
    ) => await client.config.updateSmtp(data),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.configDetails }),
  });
};

export const useConfigOAuth2Mutation = (client: Client) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['config', 'oauth2'],
    mutationFn: async (
      data: Parameters<(typeof client)['config']['updateOAuth2']>[0],
    ) => await client.config.updateOAuth2(data),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.configDetails }),
  });
};

export const useResendVerificationMutation = (client: Client) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['auth', 'resendVerification'],
    mutationFn: async () => await client.accounts.resendVerification(),
    onSuccess: () => queryClient.refetchQueries({ queryKey: queryKeys.me }),
  });
};

export const useCreateTableMutation = (client: Client) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['customTable', 'create'],
    mutationFn: async (
      table: Parameters<(typeof client)['tables']['create']>[0],
    ) => await client.tables.create(table),
    onSuccess: () => queryClient.refetchQueries({ queryKey: queryKeys.tables }),
  });
};

export const useCreateRowMutation = (client: Client, table: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['customRow', 'create'],
    mutationFn: async (
      data: Parameters<(typeof client)['tables']['createRow']>[1],
    ) => await client.tables.createRow(table, data),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.tableData(table) }),
  });
};

export const useUpdateRowMutation = (client: Client, table: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['customRow', 'update'],
    mutationFn: async ({
      match,
      data,
    }: {
      match: Parameters<(typeof client)['tables']['updateRow']>[1];
      data: Parameters<(typeof client)['tables']['updateRow']>[2];
    }) => await client.tables.updateRow(table, match, data),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.tableData(table) }),
  });
};

export const useDeleteRowMutation = (client: Client, table: string) => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ['customRow', 'delete'],
    mutationFn: async (
      match: Parameters<(typeof client)['tables']['deleteRow']>[1],
    ) => await client.tables.deleteRow(table, match),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.tableData(table) }),
  });
};
