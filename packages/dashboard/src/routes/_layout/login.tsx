import { zodResolver } from '@hookform/resolvers/zod';
import {
  SiDiscord,
  SiFacebook,
  SiGithub,
  SiGoogle,
  SiX,
} from '@icons-pack/react-simple-icons';
import { startAuthentication } from '@simplewebauthn/browser';
import { Link, createFileRoute, useRouter } from '@tanstack/react-router';
import { KeyRound, Loader2 } from 'lucide-react';
import { useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { toast } from 'sonner';
import * as z from 'zod';

import {
  Button,
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
  Input,
} from '~/components/ui';
import {
  queryKeys,
  useFinishLoginPasskeyMutation,
  useLoginMutation,
  useStartLoginPasskeyMutation,
} from '~/hooks';
import { providers } from '~/lib';

export const Route = createFileRoute('/_layout/login')({
  component: LoginComponent,
  validateSearch: (search) =>
    z
      .object({
        to: z.string().optional(),
      })
      .parse(search),
});

const providerIcons = {
  google: <SiGoogle className="size-4" />,
  facebook: <SiFacebook className="size-4" />,
  github: <SiGithub className="size-4" />,
  twitter: <SiX className="size-4" />,
  discord: <SiDiscord className="size-4" />,
};

const formSchema = z.object({
  email: z
    .string()
    .min(1, { message: 'Email is required' })
    .email({ message: 'Invalid email address' }),
  password: z
    .string()
    .min(8, { message: 'Password is too short' })
    .max(64, { message: 'Password is too long' }),
});

function LoginComponent() {
  const { client, queryClient } = Route.useRouteContext();
  const router = useRouter();
  const { to } = Route.useSearch();

  const { mutateAsync, isPending, isError, error } = useLoginMutation();

  const { mutateAsync: loginStartPasskeyMutateAsync } =
    useStartLoginPasskeyMutation();
  const { mutateAsync: loginFinishPasskeyMutateAsync } =
    useFinishLoginPasskeyMutation();

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      email: '',
      password: '',
    },
  });

  useEffect(() => {
    void queryClient.resetQueries({ queryKey: queryKeys.tokenRefresh });
    void queryClient.resetQueries({ queryKey: queryKeys.me });
  }, [queryClient]);

  useEffect(() => {
    if (isError)
      toast.error('Login failed', {
        description: (error as { message: string }).message,
      });
  }, [isError, error]);

  return (
    <div className="flex h-full w-screen items-center justify-center">
      <Card className="mx-6 w-full sm:m-0 sm:w-[500px]">
        <CardHeader>
          <CardTitle>Login</CardTitle>
        </CardHeader>

        <Form {...form}>
          <form
            onSubmit={(e) =>
              void form.handleSubmit(async (values) => {
                await mutateAsync(values);

                if (to) router.history.push(to);
                else await router.navigate({ to: '/dashboard' });
              })(e)
            }
          >
            <CardContent>
              <div className="flex flex-col gap-1">
                <FormField
                  name="email"
                  control={form.control}
                  render={({ field }) => (
                    <FormItem className="w-full">
                      <FormLabel>Email</FormLabel>
                      <FormControl>
                        <Input
                          type="email"
                          placeholder="Email"
                          data-form-type="email"
                          autoComplete="webauthn"
                          {...field}
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  name="password"
                  control={form.control}
                  render={({ field }) => (
                    <FormItem className="w-full">
                      <FormLabel>Password</FormLabel>
                      <FormControl>
                        <Input
                          type="password"
                          placeholder="Password"
                          data-form-type="password"
                          {...field}
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
              </div>
            </CardContent>

            <CardFooter>
              <div className="w-full">
                <Button type="submit" className="w-full" disabled={isPending}>
                  {isPending && (
                    <Loader2 className="mr-2 size-4 animate-spin" />
                  )}
                  Login
                </Button>

                <div className="relative my-4">
                  <div className="absolute inset-0 flex items-center">
                    <span className="w-full border-t" />
                  </div>
                  <div className="relative flex justify-center text-xs uppercase">
                    <span className="bg-background px-2 text-muted-foreground">
                      Or continue with
                    </span>
                  </div>
                </div>

                <div className="space-y-4">
                  <Button
                    type="button"
                    className="w-full"
                    variant="secondary"
                    onClick={() => {
                      void (async () => {
                        const options = await loginStartPasskeyMutateAsync();
                        const response = await startAuthentication(
                          options.publicKey,
                        );
                        await loginFinishPasskeyMutateAsync(response);

                        if (to) router.history.push(to);
                        else await router.navigate({ to: '/dashboard' });
                      })();
                    }}
                  >
                    {/* TODO(@samchouse): https://github.com/lucide-icons/lucide/pull/1848 */}
                    <KeyRound className="mr-2 size-4" /> Passkey
                  </Button>

                  <div className="grid w-full grid-cols-5 gap-2">
                    {providers.map((provider) => (
                      <Button
                        asChild
                        key={provider}
                        variant="outline"
                        className="w-full"
                      >
                        <a
                          href={client.accounts.loginUsingOAuth2(provider, {
                            to,
                          })}
                        >
                          {providerIcons[provider]}
                        </a>
                      </Button>
                    ))}
                  </div>
                </div>

                <Link
                  to="/register"
                  className="mt-4 block text-center text-muted-foreground text-sm underline underline-offset-4"
                >
                  Don&apos;t have an account? Register
                </Link>
              </div>
            </CardFooter>
          </form>
        </Form>
      </Card>
    </div>
  );
}
