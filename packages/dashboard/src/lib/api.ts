import axios, { AxiosError, AxiosResponse } from 'axios';

export const client = axios.create({
  baseURL: '/api',
  headers: {
    'Content-Type': 'application/json',
  },
});

export const hasAccessToken = () =>
  client.defaults.headers.common.Authorization !== undefined;

client.interceptors.response.use(
  (res: AxiosResponse) => Promise.resolve(res),
  (
    err: AxiosError<{
      success: false;
      message: string;
    }>,
  ) =>
    err.response?.data
      ? Promise.reject(err.response.data)
      : Promise.reject(err),
);

export const providers = [
  'google',
  'facebook',
  'github',
  'twitter',
  'discord',
] as const;

export const getTableData = async <T>(table: string) => {
  const res = await client.get<{
    success: true;
    data: T[];
  }>(`/tables/${table}/rows`);
  return res.data;
};

export const postCreateRow = async (
  table: string,
  data: Record<string, unknown>,
) => {
  const res = await client.post<{
    success: true;
    message: string;
  }>(`/tables/${table}/create`, data);
  return res.data;
};

export const patchUpdateRow = async (
  table: string,
  id: string,
  data: Record<string, unknown>,
) => {
  const res = await client.patch<{
    success: true;
    message: string;
  }>(`/tables/${table}/update?id=${id}`, data);
  return res.data;
};

export const deleteRow = async (table: string, id: string) => {
  const res = await client.delete<{
    success: true;
    message: string;
  }>(`/tables/${table}/delete/?id=${id}`);
  return res.data;
};
