import { ResponseError } from '@adrastos/lib';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useRouteContext } from '@tanstack/react-router';
import { toast } from 'sonner';

import { queryKeys } from './queries';

export const useRegisterMutation = () => {
  const { mutateAsync } = useLoginMutation();
  const { client } = useRouteContext({ strict: false });

  return useMutation({
    mutationKey: ['auth', 'register'],
    mutationFn: async (
      data: Parameters<(typeof client)['accounts']['register']>[0],
    ) => await client.accounts.register(data),
    onSuccess: async (_, vars) => mutateAsync(vars),
  });
};

export const useLoginMutation = () => {
  const queryClient = useQueryClient();
  const { client } = useRouteContext({ strict: false });

  return useMutation({
    mutationKey: ['auth', 'login'],
    mutationFn: async (
      data: Parameters<(typeof client)['accounts']['login']>[0],
    ) => await client.accounts.login(data),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.tokenRefresh }),
  });
};

export const useLogoutMutation = () => {
  const queryClient = useQueryClient();
  const { client } = useRouteContext({ strict: false });

  return useMutation({
    mutationKey: ['auth', 'logout'],
    mutationFn: async () => await client.accounts.logout(),
    onSuccess: async () => {
      client.authToken = undefined;

      await queryClient.setQueryData(queryKeys.tokenRefresh, null);
      await queryClient.setQueryData(queryKeys.me, null);
    },
  });
};

export const useUpdatePasskeyMutation = () => {
  const queryClient = useQueryClient();
  const { client } = useRouteContext({ strict: false });

  return useMutation({
    mutationKey: ['auth', 'passkeys', 'update'],
    mutationFn: async ({
      id,
      body,
    }: {
      id: Parameters<(typeof client)['accounts']['updatePasskey']>[0];
      body: Parameters<(typeof client)['accounts']['updatePasskey']>[1];
    }) => await client.accounts.updatePasskey(id, body),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.passkeys }),
  });
};

export const useDeletePasskeyMutation = () => {
  const queryClient = useQueryClient();
  const { client } = useRouteContext({ strict: false });

  return useMutation({
    mutationKey: ['auth', 'passkeys', 'delete'],
    mutationFn: async (
      id: Parameters<(typeof client)['accounts']['deletePasskey']>[0],
    ) => await client.accounts.deletePasskey(id),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.passkeys }),
  });
};

export const useStartRegisterPasskeyMutation = () => {
  const { client } = useRouteContext({ strict: false });

  return useMutation({
    mutationKey: ['auth', 'passkeys', 'register', 'start'],
    mutationFn: async () => await client.accounts.startPasskeyRegistration(),
  });
};

export const useFinishRegisterPasskeyMutation = () => {
  const queryClient = useQueryClient();
  const { client } = useRouteContext({ strict: false });

  return useMutation({
    mutationKey: ['auth', 'passkeys', 'register', 'finish'],
    mutationFn: async (
      data: Parameters<
        (typeof client)['accounts']['finishPasskeyRegistration']
      >[0],
    ) => await client.accounts.finishPasskeyRegistration(data),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.passkeys }),
  });
};

export const useStartLoginPasskeyMutation = () => {
  const { client } = useRouteContext({ strict: false });

  return useMutation({
    mutationKey: ['auth', 'passkeys', 'login', 'start'],
    mutationFn: async (
      data:
        | Parameters<(typeof client)['accounts']['startPasskeyLogin']>[0]
        | void,
    ) => await client.accounts.startPasskeyLogin(data ?? undefined),
  });
};

export const useFinishLoginPasskeyMutation = () => {
  const queryClient = useQueryClient();
  const { client } = useRouteContext({ strict: false });

  return useMutation({
    mutationKey: ['auth', 'passkeys', 'login', 'finish'],
    mutationFn: async (
      data: Parameters<(typeof client)['accounts']['finishPasskeyLogin']>[0],
    ) => await client.accounts.finishPasskeyLogin(data),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.tokenRefresh }),
  });
};

export const useConfigSmtpMutation = () => {
  const queryClient = useQueryClient();
  const { client } = useRouteContext({ strict: false });

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
  const { client } = useRouteContext({ strict: false });

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
  const { client } = useRouteContext({ strict: false });

  return useMutation({
    mutationKey: ['auth', 'resendVerification'],
    mutationFn: async () => await client.accounts.resendVerification(),
    onSuccess: () => queryClient.refetchQueries({ queryKey: queryKeys.me }),
  });
};

export const useCreateTableMutation = () => {
  const queryClient = useQueryClient();
  const { client } = useRouteContext({ strict: false });

  return useMutation({
    mutationKey: ['customTable', 'create'],
    mutationFn: async (
      table: Parameters<(typeof client)['tables']['create']>[0],
    ) => await client.tables.create(table),
    onSuccess: () => queryClient.refetchQueries({ queryKey: queryKeys.tables }),
  });
};

export const useUpdateTableMutation = () => {
  const queryClient = useQueryClient();
  const { client } = useRouteContext({ strict: false });

  return useMutation({
    mutationKey: ['customTable', 'update'],
    mutationFn: async ({
      name,
      data,
    }: {
      name: Parameters<(typeof client)['tables']['update']>[0];
      data: Parameters<(typeof client)['tables']['update']>[1];
    }) => await client.tables.update(name, data),
    onSuccess: () => queryClient.refetchQueries({ queryKey: queryKeys.tables }),
  });
};

export const useDeleteTableMutation = () => {
  const queryClient = useQueryClient();
  const { client } = useRouteContext({ strict: false });

  return useMutation({
    mutationKey: ['customTable', 'delete'],
    mutationFn: async (
      table: Parameters<(typeof client)['tables']['delete']>[0],
    ) => await client.tables.delete(table),
    onSuccess: async () =>
      queryClient.refetchQueries({ queryKey: queryKeys.tables }),
  });
};

export const useCreateRowMutation = (table: string) => {
  const queryClient = useQueryClient();
  const { client } = useRouteContext({ strict: false });

  return useMutation({
    mutationKey: ['customRow', 'create'],
    mutationFn: async (
      data: Parameters<(typeof client)['tables']['createRow']>[1],
    ) => await client.tables.createRow(table, data),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.tableData(table) }),
    onError: (error: ResponseError) =>
      toast.error("Couldn't create row", {
        description:
          error.details.message ?? error.details.error ?? error.message,
      }),
  });
};

export const useUpdateRowMutation = (table: string) => {
  const queryClient = useQueryClient();
  const { client } = useRouteContext({ strict: false });

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

export const useDeleteRowMutation = (table: string) => {
  const queryClient = useQueryClient();
  const { client } = useRouteContext({ strict: false });

  return useMutation({
    mutationKey: ['customRow', 'delete'],
    mutationFn: async (
      match: Parameters<(typeof client)['tables']['deleteRow']>[1],
    ) => await client.tables.deleteRow(table, match),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.tableData(table) }),
  });
};

export const useCreateProjectMutation = (teamId: string) => {
  const queryClient = useQueryClient();
  const { client } = useRouteContext({ strict: false });

  return useMutation({
    mutationKey: ['project', 'create'],
    mutationFn: async (data: { name: string; hostnames: string[] }) =>
      await client.json({
        path: `/teams/${teamId}/projects/create`,
        method: 'POST',
        body: JSON.stringify(data),
      }),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.projects(teamId) }),
  });
};

export const useCreateTeamMutation = () => {
  const queryClient = useQueryClient();
  const { client } = useRouteContext({ strict: false });

  return useMutation({
    mutationKey: ['team', 'create'],
    mutationFn: async (name: string) =>
      await client.json({
        path: '/teams/create',
        method: 'POST',
        body: JSON.stringify({ name }),
      }),
    onSuccess: () => queryClient.refetchQueries({ queryKey: queryKeys.teams }),
  });
};

export const useDeleteUploadMutation = () => {
  const queryClient = useQueryClient();
  const { client } = useRouteContext({ strict: false });

  return useMutation({
    mutationKey: ['storage', 'delete'],
    mutationFn: async (id: string) =>
      await client.json({
        path: `/storage/delete/${id}`,
        method: 'DELETE',
        projectIdNeeded: true,
      }),
    onSuccess: () =>
      queryClient.refetchQueries({ queryKey: queryKeys.storage }),
  });
};
