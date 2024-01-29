import { Client } from '@adrastos/lib';
import { atom } from 'jotai';

export const clientAtom = atom(
  new Client(import.meta.env.VITE_BACKEND_URL ?? '', ''),
);
