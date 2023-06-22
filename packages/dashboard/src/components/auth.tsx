'use client';

import { useEffect } from 'react';

import { useTokenRefreshQuery } from '~/hooks';
import { client } from '~/lib';

export const Auth: React.FC<React.PropsWithChildren> = ({ children }) => {
  const { data } = useTokenRefreshQuery();

  useEffect(() => {
    if (data?.accessToken)
      client.defaults.headers.Authorization = `Bearer ${data.accessToken}`;
  }, [data]);

  return children;
};
