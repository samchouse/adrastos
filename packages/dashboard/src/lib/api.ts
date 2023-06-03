import axios, { AxiosError, AxiosResponse } from 'axios';

import { env } from '~/env';

const client = axios.create({
  baseURL: '/api'
});

client.interceptors.response.use(
  (res: AxiosResponse) => Promise.resolve(res),
  (
    err: AxiosError<{
      success: false;
    }>
  ) => {
    if (err.response?.data) return Promise.reject(err.response.data);

    return Promise.reject(err);
  }
);

interface SignupData {
  firstName: string;
  lastName: string;
  email: string;
  username: string;
  password: string;
}

export const postSignup = async (data: SignupData) => {
  const res = await client.post('/auth/signup', data);
  return res.data;
};

interface LoginData {
  email: string;
  password: string;
}

export const postLogin = async (data: LoginData) => {
  const res = await client.post<
    AxiosResponse<{ success: true; accessToken: string }>
  >('/auth/login', data);
  return res.data;
};

export const getLogout = async () => {
  const res = await client.get('/auth/logout');
  return res.data;
};

export const getOauth2LoginUrl = (
  provider: 'google' | 'facebook' | 'github' | 'twitter' | 'discord'
) => `${env.NEXT_PUBLIC_BACKEND_URL}/auth/oauth2/login?provider=${provider}`;
