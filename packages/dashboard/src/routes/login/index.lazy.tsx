import { zodResolver } from '@hookform/resolvers/zod';
import {
  SiDiscord,
  SiFacebook,
  SiGithub,
  SiGoogle,
  SiTwitter,
} from '@icons-pack/react-simple-icons';
import { createLazyFileRoute, Link, useNavigate } from '@tanstack/react-router';
import { useAtomValue } from 'jotai';
import { Loader2 } from 'lucide-react';
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
import { useLoginMutation } from '~/hooks';
import { providers } from '~/lib';
import { clientAtom } from '~/lib/state';

export const Route = createLazyFileRoute('/login/')({
  component: LoginComponent,
});

const providerIcons = {
  google: <SiGoogle className="h-4 w-4" />,
  facebook: <SiFacebook className="h-4 w-4" />,
  github: <SiGithub className="h-4 w-4" />,
  twitter: <SiTwitter className="h-4 w-4" />,
  discord: <SiDiscord className="h-4 w-4" />,
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
  const client = useAtomValue(clientAtom);

  const navigate = useNavigate();
  const { to } = Route.useSearch();

  const { mutateAsync, isPending, isError, error } = useLoginMutation();

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      email: '',
      password: '',
    },
  });

  useEffect(() => {
    if (isError)
      toast('Login failed', {
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
                await navigate({
                  to: to ?? '/dashboard',
                });
              })(e)
            }
          >
            <CardContent>
              <div className="flex flex-col gap-1">
                <FormField
                  control={form.control}
                  name="email"
                  render={({ field }) => (
                    <FormItem className="w-full">
                      <FormLabel>Email</FormLabel>
                      <FormControl>
                        <Input type="email" placeholder="Email" {...field} />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name="password"
                  render={({ field }) => (
                    <FormItem className="w-full">
                      <FormLabel>Password</FormLabel>
                      <FormControl>
                        <Input
                          type="password"
                          placeholder="Password"
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
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  )}
                  Login
                </Button>

                <div className="relative my-4">
                  <div className="absolute inset-0 flex items-center">
                    <span className="w-full border-t"></span>
                  </div>
                  <div className="relative flex justify-center text-xs uppercase">
                    <span className="bg-background text-muted-foreground px-2">
                      Or continue with
                    </span>
                  </div>
                </div>

                <div className="grid w-full grid-cols-5 gap-2">
                  {providers.map((provider) => (
                    <Button
                      key={provider}
                      asChild
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

                <Link
                  to="/signup"
                  className="text-muted-foreground mt-4 block text-center text-sm underline underline-offset-4"
                >
                  Don&apos;t have an account? Sign Up
                </Link>
              </div>
            </CardFooter>
          </form>
        </Form>
      </Card>
    </div>
  );
}
