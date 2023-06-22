import axios, { AxiosError, AxiosResponse } from 'axios';

import { env } from '~/env';
import { User } from '~/types';

export const client = axios.create({
  baseURL: '/api',
  headers: {
    'Content-Type': 'application/json'
  }
});

client.interceptors.response.use(
  (res: AxiosResponse) => Promise.resolve(res),
  (
    err: AxiosError<{
      success: false;
      message: string;
    }>
  ) =>
    err.response?.data ? Promise.reject(err.response.data) : Promise.reject(err)
);

export const getMe = async () => {
  const res = await client.get<{
    success: true;
    user: User;
  }>('/me');
  return res.data;
};

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
  const res = await client.post<{ success: true; accessToken: string }>(
    '/auth/login',
    data
  );
  return res.data;
};

export const getLogout = async () => {
  const res = await client.get('/auth/logout');
  return res.data;
};

export const getTokenRefresh = async () => {
  const res = await client.get<{ success: true; accessToken: string }>(
    '/auth/token/refresh'
  );
  return res.data;
};

export const providers = [
  'google',
  'facebook',
  'github',
  'twitter',
  'discord'
] as const;

export const getOauth2LoginUrl = (
  provider: (typeof providers)[number],
  auth?: string,
  to?: string
) =>
  `${env.NEXT_PUBLIC_BACKEND_URL}/auth/oauth2/login?provider=${provider}${
    auth ? `&auth=${auth}` : ''
  }${to ? `&to=${to}` : ''}`;

export const getConfigDetails = async () => {
  const res = await client.get<{
    success: true;
    smtpConfig: {
      host: string;
      port: number;
      username: string;
      senderName: string;
      senderEmail: string;
    };
    oauth2Config: {
      google: {
        clientId: string;
        clientSecret: string;
      } | null;
      facebook: {
        clientId: string;
        clientSecret: string;
      } | null;
      github: {
        clientId: string;
        clientSecret: string;
      } | null;
      twitter: {
        clientId: string;
        clientSecret: string;
      } | null;
      discord: {
        clientId: string;
        clientSecret: string;
      } | null;
    };
  }>('/config/details');
  return res.data;
};

interface ConfigSmtpData {
  host: string;
  port: number;
  username: string;
  password: string | null;
  senderName: string;
  senderEmail: string;
}

export const postConfigSmtp = async (data: ConfigSmtpData | null) => {
  const res = await client.post('/config/smtp', data);
  return res.data;
};

interface ConfigOAuth2Data {
  google: {
    clientId: string;
    clientSecret: string;
  } | null;
  facebook: {
    clientId: string;
    clientSecret: string;
  } | null;
  github: {
    clientId: string;
    clientSecret: string;
  } | null;
  twitter: {
    clientId: string;
    clientSecret: string;
  } | null;
  discord: {
    clientId: string;
    clientSecret: string;
  } | null;
}

export const postConfigOAuth2 = async (data: ConfigOAuth2Data) => {
  const res = await client.post('/config/oauth2', data);
  return res.data;
};
