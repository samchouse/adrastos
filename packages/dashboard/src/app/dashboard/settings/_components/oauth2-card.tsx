import { zodResolver } from '@hookform/resolvers/zod';
import { Loader2 } from 'lucide-react';
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
import { useConfigOAuth2Mutation } from '~/hooks/mutations';
import { useConfigDetailsQuery } from '~/hooks/queries';
import { providers } from '~/lib';

const providerNames = {
  google: 'Google',
  facebook: 'Facebook',
  github: 'GitHub',
  twitter: 'Twitter',
  discord: 'Discord',
};

const formSchema = z.object({
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

export const OAuth2Card: React.FC = () => {
  const [enabled, setEnabled] = useState({
    google: false,
    facebook: false,
    github: false,
    twitter: false,
    discord: false,
  });

  const { data, isLoading } = useConfigDetailsQuery();
  const { mutate, isPending: mutationIsPending } = useConfigOAuth2Mutation();

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
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
