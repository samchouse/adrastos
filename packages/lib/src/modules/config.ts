import { BaseModule } from './util';

interface SmtpConfig {
  host: string;
  port: number;
  username: string;
  senderName: string;
  senderEmail: string;
}

interface OAuth2Config {
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

export class ConfigModule extends BaseModule {
  public async details() {
    return this.client.json<{
      smtpConfig: SmtpConfig;
      oauth2Config: OAuth2Config;
    }>({
      path: '/config/details',
      method: 'GET',
    });
  }

  public async updateSmtp(
    body: (SmtpConfig & { password: string | null }) | null,
  ) {
    return this.client.json<SmtpConfig>({
      path: '/config/smtp',
      method: 'POST',
      body: JSON.stringify(body),
    });
  }

  public async updateOAuth2(body: OAuth2Config) {
    return this.client.json<OAuth2Config>({
      path: '/config/oauth2',
      method: 'POST',
      body: JSON.stringify(body),
    });
  }
}
