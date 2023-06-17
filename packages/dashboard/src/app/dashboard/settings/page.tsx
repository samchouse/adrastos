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
  Switch
} from '~/components';
import { useConfigSmtpMutation } from '~/hooks/mutations';
import { useConfigDetailsQuery } from '~/hooks/queries';

const formSchema = z.object({
  host: z.string().nonempty('Host is required'),
  port: z.coerce
    .number()
    .int("Port can't contain decimals")
    .positive('Port must be positive'),
  username: z.string().nonempty('Username is required'),
  password: z.string().optional(),
  senderName: z.string().nonempty('Sender name is required'),
  senderEmail: z.string().email({ message: 'Invalid email address' })
});

const Page: React.FC = () => {
  const [smtpEnabled, setSmtpEnabled] = useState(false);
  const [editingPassword, setEditingPassword] = useState(true);
  const { data: configData, isLoading } = useConfigDetailsQuery();
  const { mutate, isLoading: mutationIsLoading } = useConfigSmtpMutation();

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      host: 'smtp.example.com',
      port: 587,
      username: '',
      password: '',
      senderName: 'Adrastos',
      senderEmail: 'no-reply@example.com'
    }
  });

  useEffect(() => {
    if (!configData?.smtpConfig) return;

    form.reset(configData.smtpConfig);
    setEditingPassword(false);
    setSmtpEnabled(true);
  }, [configData?.smtpConfig, form]);

  return (
    <>
      <Card>
        <div className="flex flex-row items-center justify-between p-6">
          <CardHeader className="p-0">
            <CardTitle>SMTP Config</CardTitle>
            <CardDescription>
              Configure your SMTP settings to send emails.
            </CardDescription>
          </CardHeader>

          <Switch
            checked={smtpEnabled}
            onCheckedChange={() => setSmtpEnabled((v) => !v)}
          />
        </div>

        <Form {...form}>
          <form
            onSubmit={form.handleSubmit((values) =>
              mutate(smtpEnabled ? values : undefined)
            )}
          >
            {smtpEnabled && (
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
                            value={editingPassword ? field.value : '********'}
                            disabled={
                              !!configData?.smtpConfig && !editingPassword
                            }
                            {...(!editingPassword && {
                              rightAdornment: (
                                <button
                                  type="button"
                                  onClick={() => {
                                    setEditingPassword(true);
                                    form.setValue('password', '');

                                    setTimeout(() => {
                                      form.setFocus('password');
                                    }, 0);
                                  }}
                                  className="absolute right-3 top-1/2 -translate-y-1/2 disabled:cursor-not-allowed disabled:opacity-50"
                                >
                                  <Edit2 className="text-primary h-4 w-4" />
                                </button>
                              )
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
                  !form.formState.isDirty &&
                  smtpEnabled &&
                  !!configData?.smtpConfig
                }
                onClick={() => {
                  if (configData?.smtpConfig) {
                    setSmtpEnabled(true);
                    setEditingPassword(false);
                  } else setSmtpEnabled(false);

                  form.reset();
                }}
              >
                Cancel
              </Button>
              <Button
                type="submit"
                disabled={
                  (!form.formState.isDirty &&
                    smtpEnabled &&
                    !!configData?.smtpConfig) ||
                  isLoading ||
                  mutationIsLoading
                }
              >
                {(isLoading || mutationIsLoading) && (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                )}
                Save
              </Button>
            </CardFooter>
          </form>
        </Form>
      </Card>
    </>
  );
};

export default Page;
