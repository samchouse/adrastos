import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

export const Route = createFileRoute('/_layout/login/')({
  validateSearch: (search) =>
    z
      .object({
        to: z.string().optional(),
      })
      .parse(search),
});
