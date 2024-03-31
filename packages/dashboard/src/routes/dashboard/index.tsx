import { createFileRoute, redirect } from '@tanstack/react-router';

import { teamsQueryOptions } from '~/hooks';

export const Route = createFileRoute('/dashboard/')({
  beforeLoad: async ({ context: { client, queryClient } }) => {
    const teams = await queryClient.ensureQueryData(teamsQueryOptions(client));
    throw redirect({
      to: '/dashboard/teams/$teamId',
      params: { teamId: teams[0].id },
      replace: true,
    });
  },
});
