'use client';

import { useAtomValue } from 'jotai';
import { useEffect } from 'react';

import { useTokenRefreshQuery } from '~/hooks';
import { client as oldClient } from '~/lib';
import { clientAtom } from '~/lib/state';

export const Auth: React.FC<React.PropsWithChildren> = ({ children }) => {
  const client = useAtomValue(clientAtom);

  const { data: accessToken } = useTokenRefreshQuery();

  useEffect(() => {
    if (accessToken) {
      oldClient.defaults.headers.common.Authorization = `Bearer ${accessToken}`;
      client.authToken = accessToken;
    }
  }, [accessToken, client]);

  return children;
};
