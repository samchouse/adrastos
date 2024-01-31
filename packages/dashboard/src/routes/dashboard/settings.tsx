import { zodResolver } from '@hookform/resolvers/zod';
import { createFileRoute } from '@tanstack/react-router';
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
import {
  useConfigDetailsQuery,
  useConfigOAuth2Mutation,
  useConfigSmtpMutation,
} from '~/hooks';
import { providers } from '~/lib';

export const Route = createFileRoute('/dashboard/settings')({
  component: SettingsComponent,
});

const smtpFormSchema = z.object({
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

  const form = useForm<z.infer<typeof smtpFormSchema>>({
    resolver: zodResolver(smtpFormSchema),
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
          onSubmit={(e) =>
            void form.handleSubmit((values) => mutate(enabled ? values : null))(
              e,
            )
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

const providerNames = {
  google: 'Google',
  facebook: 'Facebook',
  github: 'GitHub',
  twitter: 'Twitter',
  discord: 'Discord',
};

const oauth2FormSchema = z.object({
  google: z
    .object({
      clientId: z.string().min(1, 'Client ID is required'),
      clientSecret: z.string().min(1, 'Client secret is required'),
    })
    .nullable(),
  facebook: z
    .object({
      clientId: z.string().min(1, 'Client ID is required'),
      clientSecret: z.string().min(1, 'Client secret is required'),
    })
    .nullable(),
  github: z
    .object({
      clientId: z.string().min(1, 'Client ID is required'),
      clientSecret: z.string().min(1, 'Client secret is required'),
    })
    .nullable(),
  twitter: z
    .object({
      clientId: z.string().min(1, 'Client ID is required'),
      clientSecret: z.string().min(1, 'Client secret is required'),
    })
    .nullable(),
  discord: z
    .object({
      clientId: z.string().min(1, 'Client ID is required'),
      clientSecret: z.string().min(1, 'Client secret is required'),
    })
    .nullable(),
});

const OAuth2Card: React.FC = () => {
  const [enabled, setEnabled] = useState({
    google: false,
    facebook: false,
    github: false,
    twitter: false,
    discord: false,
  });

  const { data, isLoading } = useConfigDetailsQuery();
  const { mutate, isPending: mutationIsPending } = useConfigOAuth2Mutation();

  const form = useForm<z.infer<typeof oauth2FormSchema>>({
    resolver: zodResolver(oauth2FormSchema),
    defaultValues: {
      google: null,
      facebook: null,
      github: null,
      twitter: null,
      discord: null,
    },
    mode: 'onChange',
  });

  useEffect(() => {
    if (!data?.oauth2Config) return;

    form.reset(data.oauth2Config);
    providers.forEach((provider) => {
      if (data.oauth2Config[provider])
        setEnabled((v) => ({ ...v, [provider]: true }));
    });
  }, [data?.oauth2Config, form]);

  useEffect(() => {
    providers.forEach((provider) => {
      const values = form.getValues(provider);
      if (enabled[provider] && !values)
        form.setValue(provider, { clientId: '', clientSecret: '' });
      else if (!enabled[provider] && !values?.clientId && !values?.clientSecret)
        form.setValue(provider, null);
    });
  }, [form, enabled]);

  return (
    <Card>
      <div className="flex flex-row items-center justify-between p-6">
        <CardHeader className="p-0">
          <CardTitle>OAuth2 Config</CardTitle>
          <CardDescription>
            Configure your OAuth2 settings to enable login with other providers.
          </CardDescription>
        </CardHeader>
      </div>

      <Form {...form}>
        <form
          onSubmit={(e) =>
            void form.handleSubmit((values) =>
              mutate(
                providers.reduce(
                  (acc, provider) => ({
                    ...acc,
                    [provider]: enabled[provider] ? values[provider] : null,
                  }),
                  {} as Record<
                    (typeof providers)[number],
                    { clientId: string; clientSecret: string } | null
                  >,
                ),
              ),
            )(e)
          }
        >
          <CardContent>
            <div className="flex flex-col gap-y-3">
              {providers.map((provider) => (
                <div key={provider}>
                  <div className="flex items-center justify-between">
                    <h3 className="text-base font-medium">
                      {providerNames[provider]}
                    </h3>

                    <Switch
                      checked={enabled[provider]}
                      onCheckedChange={() =>
                        setEnabled((v) => ({ ...v, [provider]: !v[provider] }))
                      }
                    />
                  </div>

                  {enabled[provider] && form.watch(provider) !== null && (
                    <div className="mt-1 flex gap-x-4">
                      <FormField
                        control={form.control}
                        name={`${provider}.clientId`}
                        render={({ field }) => (
                          <FormItem className="w-full">
                            <FormLabel>Client ID</FormLabel>
                            <FormControl>
                              <Input
                                {...field}
                                data-form-type="other"
                                placeholder="Client ID"
                              />
                            </FormControl>
                            <FormMessage />
                          </FormItem>
                        )}
                      />
                      <FormField
                        control={form.control}
                        name={`${provider}.clientSecret`}
                        render={({ field }) => (
                          <FormItem className="w-full">
                            <FormLabel>Client Secret</FormLabel>
                            <FormControl>
                              <Input
                                {...field}
                                data-form-type="other"
                                placeholder="Client Secret"
                              />
                            </FormControl>
                            <FormMessage />
                          </FormItem>
                        )}
                      />
                    </div>
                  )}
                </div>
              ))}
            </div>
          </CardContent>

          <CardFooter className="justify-end gap-x-2">
            <Button
              type="reset"
              variant="ghost"
              disabled={
                (!form.formState.isDirty &&
                  providers.every(
                    (provider) =>
                      enabled[provider] === !!data?.oauth2Config[provider],
                  )) ||
                isLoading ||
                mutationIsPending
              }
              onClick={() => {
                providers.forEach((provider) => {
                  setEnabled((v) => ({
                    ...v,
                    [provider]: !!data?.oauth2Config[provider],
                  }));
                });

                form.reset();
              }}
            >
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={
                !form.formState.isValid ||
                (!form.formState.isDirty &&
                  providers.every(
                    (provider) =>
                      enabled[provider] === !!data?.oauth2Config[provider],
                  )) ||
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

function SettingsComponent() {
  return (
    <div className="flex flex-col gap-y-5 p-5">
      <SmtpCard />
      <OAuth2Card />
    </div>
  );
}
