// @ts-nocheck

import {
  SiDiscord,
  SiFacebook,
  SiGithub,
  SiGoogle,
  SiTwitter,
} from '@icons-pack/react-simple-icons';
import { useAtomValue } from 'jotai';
import Link from 'next/link';
import { usePathname } from 'next/navigation';

import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '~/components';
import { useTokenRefreshQuery } from '~/hooks';
import { providers } from '~/lib';
import { clientAtom } from '~/lib/state';

const providerIcons = {
  google: <SiGoogle className="h-4 w-4" />,
  facebook: <SiFacebook className="h-4 w-4" />,
  github: <SiGithub className="h-4 w-4" />,
  twitter: <SiTwitter className="h-4 w-4" />,
  discord: <SiDiscord className="h-4 w-4" />,
};

export const OAuth2Card: React.FC = () => {
  const pathname = usePathname();
  const client = useAtomValue(clientAtom);
  const { data: accessToken } = useTokenRefreshQuery();

  return (
    <Card>
      <CardHeader>
        <CardTitle>OAuth2</CardTitle>
        <CardDescription>
          Enable login with different providers for this account
        </CardDescription>
      </CardHeader>

      <CardContent>
        <div className="grid w-full grid-cols-5 gap-2">
          {providers.map((provider) => (
            <Button key={provider} asChild variant="outline" className="w-full">
              <Link
                href={client.accounts.loginUsingOAuth2(provider, {
                  auth: accessToken,
                  to: pathname,
                })}
              >
                {providerIcons[provider]}
              </Link>
            </Button>
          ))}
        </div>
      </CardContent>
    </Card>
  );
};
