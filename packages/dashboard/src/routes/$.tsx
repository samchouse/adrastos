import { createFileRoute, notFound } from '@tanstack/react-router';

import { NotFound } from '~/components';

export const Route = createFileRoute('/$')({
  notFoundComponent: NotFound,
  beforeLoad: () => {
    throw notFound();
  },
});
