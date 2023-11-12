import { Client } from '@adrastos/lib';
import { atom } from 'jotai';

export const clientAtom = atom(new Client('/api', 'dsf'));
