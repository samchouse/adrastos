import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';
import { createRouter, RouterProvider } from '@tanstack/react-router';
import { StrictMode } from 'react';
import ReactDOM from 'react-dom/client';

import { Toaster } from '~/components/ui';

import { routeTree } from './routeTree.gen';

import './index.css';

import { Client } from '@adrastos/lib';

const queryClient = new QueryClient();
const router = createRouter({
  routeTree,
  context: {
    queryClient,
    client: new Client(import.meta.env.VITE_BACKEND_URL ?? ''),
  },
  Wrap: ({ children }) => (
    <QueryClientProvider client={queryClient}>
      <Toaster closeButton position="bottom-center" />
      <ReactQueryDevtools position="bottom" buttonPosition="bottom-left" />

      {children}
    </QueryClientProvider>
  ),
});

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <RouterProvider router={router} />
  </StrictMode>,
);
