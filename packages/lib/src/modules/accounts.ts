import { User } from '../types';
import { merge } from '../util';
import { BaseModule } from './util';

export class AccountsModule extends BaseModule {
  public async signup(body: {
    firstName: string;
    lastName: string;
    email: string;
    username: string;
    password: string;
  }) {
    return this.client.request<User>({
      path: '/auth/signup',
      method: 'POST',
      body: JSON.stringify(body),
    });
  }

  public async login(body: { email: string; password: string }) {
    return this.client.request<User>({
      path: '/auth/login',
      method: 'POST',
      options: {
        credentials: 'include',
      },
      body: JSON.stringify(body),
    });
  }

  public loginUsingOAuth2(
    provider: 'google' | 'facebook' | 'github' | 'twitter' | 'discord',
    { auth, to }: { auth?: string; to?: string } = {},
  ) {
    return this.client
      .buildUrl(
        merge(
          `/auth/oauth2/login?provider=${provider}`,
          to && `&to=${to}`,
          auth && `&auth=${auth}`,
        ),
      )
      .toString();
  }

  public async logout() {
    return this.client.request<null>({
      path: '/auth/logout',
      method: 'GET',
      options: {
        credentials: 'include',
      },
    });
  }

  public async resendVerification() {
    return this.client.request<null>({
      path: '/auth/resend-verification',
      method: 'POST',
    });
  }

  public async enableMfa() {
    return this.client.request<{ secret: string; qrCode: string }>({
      path: '/auth/mfa/enable',
      method: 'GET',
    });
  }

  public async confirmMfa(body: { code: string }) {
    return this.client.request<string[]>({
      path: '/auth/mfa/confirm',
      method: 'POST',
      body: JSON.stringify(body),
    });
  }

  public async verifyMfa(body: { code: string }) {
    return this.client.request<User>({
      path: '/auth/mfa/verify',
      method: 'POST',
      body: JSON.stringify(body),
    });
  }

  public async disableMfa(body: { code: string }) {
    return this.client.request<null>({
      path: '/auth/mfa/disable',
      method: 'POST',
      body: JSON.stringify(body),
    });
  }

  public async regenerateMfaCodes(body: { code: string }) {
    return this.client.request<string[]>({
      path: '/auth/mfa/codes/regenerate',
      method: 'POST',
      body: JSON.stringify(body),
    });
  }

  public async refreshToken() {
    return this.client.request<string>({
      path: '/auth/token/refresh',
      method: 'GET',
      options: {
        credentials: 'include',
      },
    });
  }

  public async currentUser() {
    return this.client.request<User>({
      path: '/me',
      method: 'GET',
    });
  }
}
