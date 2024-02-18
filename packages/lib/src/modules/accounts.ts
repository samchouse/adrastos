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
    return this.client.request({
      path: '/auth/logout',
      method: 'GET',
      options: {
        credentials: 'include',
      },
    });
  }

  public async resendVerification() {
    return this.client.request({
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
    return this.client.request({
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

  public async listPasskeys() {
    return this.client.request<
      {
        id: string;
        name: string;
        lastUsed?: Date;
        createdAt: Date;
        updatedAt?: Date;
      }[]
    >({
      path: '/auth/passkeys/list',
      method: 'GET',
    });
  }

  public async updatePasskey(id: string, body: { name: string }) {
    return this.client.request({
      path: `/auth/passkeys/update/${id}`,
      method: 'POST',
      body: JSON.stringify(body),
    });
  }

  public async deletePasskey(id: string) {
    return this.client.request({
      path: `/auth/passkeys/delete/${id}`,
      method: 'DELETE',
    });
  }

  public async startPasskeyRegistration() {
    return this.client.request<{
      publicKey: {
        rp: PublicKeyCredentialRpEntity;
        user: {
          id: string;
          name: string;
          displayName: string;
        };
        challenge: string;
        pubKeyCredParams: PublicKeyCredentialParameters[];
        timeout?: number;
        excludeCredentials?: {
          id: string;
          type: PublicKeyCredentialType;
          transports?: (
            | 'ble'
            | 'cable'
            | 'hybrid'
            | 'internal'
            | 'nfc'
            | 'smart-card'
            | 'usb'
          )[];
        }[];
        authenticatorSelection?: AuthenticatorSelectionCriteria;
        attestation?: AttestationConveyancePreference;
        extensions?: AuthenticationExtensionsClientInputs;
      };
    }>({
      path: '/auth/passkeys/register/start',
      method: 'POST',
      options: {
        credentials: 'include',
      },
    });
  }

  public async finishPasskeyRegistration(body: {
    name: string;
    passkey: {
      id: string;
      rawId: string;
      response: {
        clientDataJSON: string;
        attestationObject: string;
        authenticatorData?: string;
        transports?: (
          | 'ble'
          | 'cable'
          | 'hybrid'
          | 'internal'
          | 'nfc'
          | 'smart-card'
          | 'usb'
        )[];
        publicKeyAlgorithm?: COSEAlgorithmIdentifier;
        publicKey?: string;
      };
      authenticatorAttachment?: AuthenticatorAttachment;
      clientExtensionResults: AuthenticationExtensionsClientOutputs;
      type: PublicKeyCredentialType;
    };
  }) {
    return this.client.request({
      path: '/auth/passkeys/register/finish',
      method: 'POST',
      options: {
        credentials: 'include',
      },
      body: JSON.stringify(body),
    });
  }

  public async startPasskeyLogin(body?: { id: string }) {
    return this.client.request<{
      publicKey: {
        challenge: string;
        timeout?: number;
        rpId?: string;
        allowCredentials?: {
          id: string;
          type: PublicKeyCredentialType;
          transports?: (
            | 'ble'
            | 'cable'
            | 'hybrid'
            | 'internal'
            | 'nfc'
            | 'smart-card'
            | 'usb'
          )[];
        }[];
        userVerification?: UserVerificationRequirement;
        extensions?: AuthenticationExtensionsClientInputs;
      };
    }>({
      path: '/auth/passkeys/login/start',
      method: 'POST',
      body: JSON.stringify(body ?? null),
      options: {
        credentials: 'include',
      },
    });
  }

  public async finishPasskeyLogin(body: {
    id: string;
    rawId: string;
    response: {
      clientDataJSON: string;
      authenticatorData: string;
      signature: string;
      userHandle?: string;
    };
    authenticatorAttachment?: AuthenticatorAttachment;
    clientExtensionResults: AuthenticationExtensionsClientOutputs;
    type: PublicKeyCredentialType;
  }) {
    return this.client.request({
      path: '/auth/passkeys/login/finish',
      method: 'POST',
      options: {
        credentials: 'include',
      },
      body: JSON.stringify(body),
    });
  }
}
