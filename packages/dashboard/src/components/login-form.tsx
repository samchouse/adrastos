'use client';

import { zodResolver } from '@hookform/resolvers/zod';
import {
  SiDiscord,
  SiFacebook,
  SiGithub,
  SiGoogle,
  SiTwitter
} from '@icons-pack/react-simple-icons';
import Link from 'next/link';
import { useForm } from 'react-hook-form';
import * as z from 'zod';

import {
  Button,
  CardContent,
  CardFooter,
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
  Input
} from '~/components/ui';
import { useLoginMutation } from '~/hooks/mutations';
import { getOauth2LoginUrl } from '~/lib';

const formSchema = z.object({
  email: z
    .string()
    .nonempty({ message: 'Email is required' })
    .email({ message: 'Invalid email address' }),
  password: z
    .string()
    .nonempty({ message: 'Password is required' })
    .min(8, { message: 'Password is too short' })
    .max(64, { message: 'Password is too long' })
});

export const LoginForm: React.FC = () => {
  const { mutate } = useLoginMutation();

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      email: '',
      password: ''
    }
  });

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit((values) => mutate(values))}>
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
                    <Input type="password" placeholder="Password" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
          </div>
        </CardContent>

        <CardFooter>
          <div className="w-full">
            <Button type="submit" className="w-full">
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
              <Button asChild variant="outline" className="w-full">
                <Link href={getOauth2LoginUrl('google')}>
                  <SiGoogle className="h-4 w-4" />
                </Link>
              </Button>
              <Button asChild variant="outline" className="w-full">
                <Link href={getOauth2LoginUrl('facebook')}>
                  <SiFacebook className="h-4 w-4" />
                </Link>
              </Button>
              <Button asChild variant="outline" className="w-full">
                <Link href={getOauth2LoginUrl('github')}>
                  <SiGithub className="h-4 w-4" />
                </Link>
              </Button>
              <Button asChild variant="outline" className="w-full">
                <Link href={getOauth2LoginUrl('twitter')}>
                  <SiTwitter className="h-4 w-4" />
                </Link>
              </Button>
              <Button asChild variant="outline" className="w-full">
                <Link href={getOauth2LoginUrl('discord')}>
                  <SiDiscord className="h-4 w-4" />
                </Link>
              </Button>
            </div>

            <Link
              href="/signup"
              className="text-muted-foreground mt-4 block text-center text-sm underline underline-offset-4"
            >
              Don&apos;t have an account? Sign Up
            </Link>
          </div>
        </CardFooter>
      </form>
    </Form>
  );
};
