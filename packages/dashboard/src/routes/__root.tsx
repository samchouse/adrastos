import { useQuery } from '@tanstack/react-query';
import {
  createRootRouteWithContext,
  Outlet,
  redirect,
} from '@tanstack/react-router';
import React, { Suspense, useEffect } from 'react';

import { meQueryOptions, tokenRefreshQueryOptions } from '~/hooks';
import { RouterContext } from '~/types';

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootComponent,
  loader: async ({ context: { client, queryClient } }) => ({
    accessToken: await queryClient
      .ensureQueryData(tokenRefreshQueryOptions(client))
      .catch(() => undefined),
    user: await queryClient
      .ensureQueryData(meQueryOptions(client))
      .catch(() => undefined),
  }),
  beforeLoad: async ({ location, context: { client, queryClient } }) => {
    client.authToken = await queryClient
      .ensureQueryData(tokenRefreshQueryOptions(client))
      .catch(() => undefined);

    if (
      ['/', '/login', '/register'].includes(location.pathname) &&
      client.hasAuthToken
    )
      throw redirect({
        to: '/dashboard',
      });

    if (location.pathname.startsWith('/dashboard') && !client.hasAuthToken)
      throw redirect({
        to: '/login',
        search: {
          to: location.pathname,
        },
      });
  },
});

const TanStackRouterDevtools = import.meta.env.PROD
  ? () => null
  : React.lazy(() =>
      import('@tanstack/router-devtools').then((res) => ({
        default: res.TanStackRouterDevtools,
      })),
    );

function RootComponent() {
  const { client } = Route.useRouteContext();

  const { data: accessToken } = useQuery(tokenRefreshQueryOptions(client));

  useEffect(() => {
    client.authToken = accessToken;
  }, [accessToken, client]);

  return (
    <div className="bg-background text-primary flex h-screen flex-col font-['Work_Sans']">
      <Outlet />

      <Suspense fallback={null}>
        <TanStackRouterDevtools position="bottom-right" />
      </Suspense>
    </div>
  );
}
