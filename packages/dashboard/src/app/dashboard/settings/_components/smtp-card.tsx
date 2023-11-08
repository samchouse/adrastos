'use client';

import { zodResolver } from '@hookform/resolvers/zod';
import { Edit2, Loader2 } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';

import {
  Button,
  Card,
  CardContent,
  CardDescription,
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
  Switch,
} from '~/components';
import { useConfigSmtpMutation } from '~/hooks/mutations';
import { useConfigDetailsQuery } from '~/hooks/queries';

const formSchema = z.object({
  host: z.string().min(1, 'Host is required'),
  port: z.coerce
    .number()
    .int("Port can't contain decimals")
    .positive('Port must be positive'),
  username: z.string().min(1, 'Username is required'),
  password: z
    .string()
    .nullable()
    .refine((password) => password !== '', 'Password is required'),
  senderName: z.string().min(1, 'Sender name is required'),
  senderEmail: z.string().email({ message: 'Invalid email address' }),
});

export const SmtpCard: React.FC = () => {
  const [enabled, setEnabled] = useState(false);

  const { data, isLoading } = useConfigDetailsQuery();
  const { mutate, isPending: mutationIsPending } = useConfigSmtpMutation();

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      host: 'smtp.example.com',
      port: 587,
      username: '',
      password: '',
      senderName: 'Adrastos',
      senderEmail: 'no-reply@example.com',
    },
  });

  useEffect(() => {
    if (data?.smtpConfig) setEnabled(true);

    form.reset(
      data?.smtpConfig
        ? { ...data.smtpConfig, password: null }
        : {
            host: 'smtp.example.com',
            port: 587,
            username: '',
            password: '',
            senderName: 'Adrastos',
            senderEmail: 'no-reply@example.com',
          },
    );
  }, [data?.smtpConfig, form]);

  return (
    <Card>
      <div className="flex flex-row items-center justify-between p-6">
        <CardHeader className="p-0">
          <CardTitle>SMTP Config</CardTitle>
          <CardDescription>
            Configure your SMTP settings to send emails.
          </CardDescription>
        </CardHeader>

        <Switch
          checked={enabled}
          onCheckedChange={() => setEnabled((v) => !v)}
        />
      </div>

      <Form {...form}>
        <form
          onSubmit={
            void form.handleSubmit((values) => mutate(enabled ? values : null))
          }
        >
          {enabled && (
            <CardContent>
              <div className="grid grid-cols-2 gap-x-4 gap-y-1">
                <FormField
                  control={form.control}
                  name="senderName"
                  render={({ field }) => (
                    <FormItem className="w-full">
                      <FormLabel>Sender Name</FormLabel>
                      <FormControl>
                        <Input {...field} placeholder="Sender Name" />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name="senderEmail"
                  render={({ field }) => (
                    <FormItem className="w-full">
                      <FormLabel>Sender Email</FormLabel>
                      <FormControl>
                        <Input
                          {...field}
                          type="email"
                          placeholder="Sender Email"
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name="host"
                  render={({ field }) => (
                    <FormItem className="w-full">
                      <FormLabel>Host</FormLabel>
                      <FormControl>
                        <Input {...field} placeholder="Host" />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name="port"
                  render={({ field }) => (
                    <FormItem className="w-full">
                      <FormLabel>Port</FormLabel>
                      <FormControl>
                        <Input {...field} type="number" placeholder="Port" />
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
                        <Input {...field} placeholder="Username" />
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
                          {...field}
                          type="password"
                          placeholder="Password"
                          value={field.value ?? '********'}
                          disabled={!!data?.smtpConfig && field.value === null}
                          {...(field.value === null && {
                            rightAdornment: (
                              <button
                                type="button"
                                onClick={() => {
                                  form.setValue('password', '');

                                  setTimeout(() => {
                                    form.setFocus('password');
                                  }, 0);
                                }}
                                className="absolute right-3 top-1/2 -translate-y-1/2 disabled:cursor-not-allowed disabled:opacity-50"
                              >
                                <Edit2 className="text-primary h-4 w-4" />
                              </button>
                            ),
                          })}
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
              </div>
            </CardContent>
          )}

          <CardFooter className="justify-end gap-x-2">
            <Button
              type="reset"
              variant="ghost"
              disabled={
                (!form.formState.isDirty &&
                  enabled === !!data?.smtpConfig &&
                  form.watch('password') === null) ||
                isLoading ||
                mutationIsPending
              }
              onClick={() => {
                if (data?.smtpConfig) {
                  setEnabled(true);
                  form.setValue('password', null);
                } else setEnabled(false);

                form.reset();
              }}
            >
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={
                (!form.formState.isDirty && enabled === !!data?.smtpConfig) ||
                isLoading ||
                mutationIsPending
              }
            >
              {(isLoading || mutationIsPending) && (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              )}
              Save
            </Button>
          </CardFooter>
        </form>
      </Form>
    </Card>
  );
};
