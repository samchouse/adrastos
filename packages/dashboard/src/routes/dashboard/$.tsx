import { createFileRoute, notFound } from '@tanstack/react-router';

import { NotFound } from '~/components';

export const Route = createFileRoute('/dashboard/$')({
  notFoundComponent: NotFound,
  beforeLoad: () => {
    throw notFound();
  },
});
