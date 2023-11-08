'use client';

import { zodResolver } from '@hookform/resolvers/zod';
import { Loader2 } from 'lucide-react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useForm } from 'react-hook-form';
import * as z from 'zod';

import {
  Button,
  CardContent,
  CardFooter,
  Checkbox,
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
  Input,
} from '~/components';
import { useSignupMutation } from '~/hooks';

const formSchema = z
  .object({
    firstName: z
      .string()
      .min(1, { message: 'First name is required' })
      .max(50, { message: 'First name must be less than 50 characters' }),
    lastName: z
      .string()
      .min(1, { message: 'Last name is required' })
      .max(50, { message: 'Last name must be less than 50 characters' }),
    email: z
      .string()
      .min(1, { message: 'Email is required' })
      .email({ message: 'Invalid email address' }),
    username: z
      .string()
      .min(5, { message: 'Username must be at least 5 characters' })
      .max(64, { message: 'Username must be less than 64 characters' }),
    password: z
      .string()
      .min(8, { message: 'Password must be at least 8 characters' })
      .max(64, { message: 'Password must be less than 64 characters' }),
    confirmPassword: z.string().min(1, { message: 'Re-enter your password' }),
    tac: z.boolean().refine((v) => v, {
      message: 'You must accept the Terms and Conditions',
    }),
  })
  .superRefine(({ password, confirmPassword }, ctx) => {
    if (password !== confirmPassword)
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        message: "The passwords don't match",
        path: ['confirmPassword'],
      });
  });

export const SignupForm: React.FC = () => {
  const router = useRouter();
  const { mutateAsync, isPending } = useSignupMutation();

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      firstName: '',
      lastName: '',
      email: '',
      username: '',
      password: '',
      tac: false,
    },
  });

  return (
    <Form {...form}>
      <form
        onSubmit={
          void form.handleSubmit(async (values) => {
            await mutateAsync(values);
            router.push('/dashboard');
          })
        }
      >
        <CardContent>
          <div className="flex flex-col gap-1">
            <div className="grid grid-cols-2 gap-x-4 gap-y-1">
              <FormField
                control={form.control}
                name="firstName"
                render={({ field }) => (
                  <FormItem className="w-full">
                    <FormLabel>First Name</FormLabel>
                    <FormControl>
                      <Input placeholder="First Name" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={form.control}
                name="lastName"
                render={({ field }) => (
                  <FormItem className="w-full">
                    <FormLabel>Last Name</FormLabel>
                    <FormControl>
                      <Input placeholder="Last Name" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={form.control}
                name="username"
                render={({ field }) => (
                  <FormItem className="w-full">
                    <FormLabel>Username</FormLabel>
                    <FormControl>
                      <Input placeholder="Username" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
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
            </div>

            <FormField
              control={form.control}
              name="password"
              render={({ field }) => (
                <FormItem className="w-full">
                  <FormLabel>Password</FormLabel>
                  <FormControl>
                    <Input
                      {...field}
                      type="password"
                      placeholder="Password"
                      data-form-type="password,new"
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="confirmPassword"
              render={({ field }) => (
                <FormItem className="w-full">
                  <FormLabel>Confirm Password</FormLabel>
                  <FormControl>
                    <Input
                      {...field}
                      type="password"
                      data-form-type="password,confirmation"
                      placeholder="Confirm Password"
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="tac"
              render={({ field }) => (
                <FormItem className="mt-3 w-full">
                  <div className="flex items-center space-x-2">
                    <FormControl>
                      <Checkbox
                        {...{ ...field, value: undefined, onChange: undefined }}
                        checked={field.value}
                        onCheckedChange={(state) => field.onChange(!!state)}
                      />
                    </FormControl>
                    <FormLabel className="cursor-pointer">
                      I accept the Terms and Conditions
                    </FormLabel>
                  </div>
                  <FormMessage />
                </FormItem>
              )}
            />
          </div>
        </CardContent>

        <CardFooter>
          <div className="w-full">
            <Button type="submit" className="w-full" disabled={isPending}>
              {isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Sign Up
            </Button>

            <Link
              href="/login"
              className="text-muted-foreground mt-5 block text-center text-sm underline underline-offset-4"
            >
              Already have an account? Login
            </Link>
          </div>
        </CardFooter>
      </form>
    </Form>
  );
};
