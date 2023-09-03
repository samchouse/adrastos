import { BaseModule } from './util';

export class AccountsModule extends BaseModule {
  public async create(body: {
    firstName: string;
    lastName: string;
    email: string;
    username: string;
    password: string;
  }) {
    this.client.request({
      path: '/auth/signup',
      method: 'POST',
      body: JSON.stringify(body),
    });
  }

  public async authWithPassword(body: {
    email: string;
    password: string;
  }): Promise<{ token: string }> {
    return this.client.request({
      path: '/auth/login',
      method: 'POST',
      body: JSON.stringify(body),
    });
  }

  public authWithOAuth2(
    provider: 'google' | 'facebook' | 'github' | 'twitter' | 'discord',
  ) {
    return this.client.buildUrl(`/auth/oauth2/${provider}`).toString();
  }
}
