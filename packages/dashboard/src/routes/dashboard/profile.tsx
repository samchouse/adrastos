import { ResponseError } from '@adrastos/lib';
import { zodResolver } from '@hookform/resolvers/zod';
import {
  SiDiscord,
  SiFacebook,
  SiGithub,
  SiGoogle,
  SiTwitter,
} from '@icons-pack/react-simple-icons';
import { startRegistration, WebAuthnError } from '@simplewebauthn/browser';
import { type RegistrationResponseJSON } from '@simplewebauthn/types';
import { useSuspenseQueries } from '@tanstack/react-query';
import { createFileRoute, useRouterState } from '@tanstack/react-router';
import { formatDistanceToNow } from 'date-fns';
import { Loader2, MoreHorizontal, Pencil, Plus, Trash2 } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { toast } from 'sonner';
import { z } from 'zod';

import {
  Alert,
  AlertDescription,
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertTitle,
  Badge,
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
  Input,
  MultiDialog,
  Table,
  TableBody,
  TableCell,
  TableRow,
} from '~/components';
import {
  meQueryOptions,
  passkeysQueryOptions,
  tokenRefreshQueryOptions,
  useDeletePasskeyMutation,
  useFinishRegisterPasskeyMutation,
  useResendVerificationMutation,
  useStartRegisterPasskeyMutation,
  useUpdatePasskeyMutation,
} from '~/hooks';
import { providers } from '~/lib';

export const Route = createFileRoute('/dashboard/profile')({
  component: ProfileComponent,
  loader: async ({ context: { queryClient, client } }) => ({
    passkeys: await queryClient.ensureQueryData(passkeysQueryOptions(client)),
  }),
});

const providerIcons = {
  google: <SiGoogle className="h-4 w-4" />,
  facebook: <SiFacebook className="h-4 w-4" />,
  github: <SiGithub className="h-4 w-4" />,
  twitter: <SiTwitter className="h-4 w-4" />,
  discord: <SiDiscord className="h-4 w-4" />,
};

const formSchema = z.object({
  name: z.string().min(1, { message: 'Name is required' }),
});

const EditPasskeyDialog: React.FC<{
  passkey: {
    id: string;
    name: string;
    lastUsed?: Date | undefined;
    createdAt: Date;
    updatedAt?: Date | undefined;
  };
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}> = ({ passkey, open, onOpenChange }) => {
  const { client } = Route.useRouteContext();

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      name: passkey.name,
    },
  });

  const { mutateAsync } = useUpdatePasskeyMutation(client);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Edit passkey</DialogTitle>
        </DialogHeader>

        <Form {...form}>
          <form
            onSubmit={(e) =>
              void form.handleSubmit(async (values) => {
                await mutateAsync({ id: passkey.id, body: values });
                onOpenChange?.(false);
              })(e)
            }
          >
            <div className="space-y-5">
              <FormField
                control={form.control}
                name="name"
                render={({ field }) => (
                  <FormItem className="w-full">
                    <FormLabel>Name</FormLabel>
                    <FormControl>
                      <Input placeholder="Name" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <DialogFooter>
                <Button type="submit">Submit</Button>
              </DialogFooter>
            </div>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
};

function ProfileComponent() {
  const routerState = useRouterState();
  const { client } = Route.useRouteContext();

  const [{ data: accessToken }, { data: user }, { data: passkeys }] =
    useSuspenseQueries({
      queries: [
        tokenRefreshQueryOptions(client),
        meQueryOptions(client),
        passkeysQueryOptions(client),
      ],
    });

  const { mutate, isPending } = useResendVerificationMutation(client);
  const { mutate: deletePasskeyMutate } = useDeletePasskeyMutation(client);
  const { mutateAsync: registerStartPasskeyMutateAsync } =
    useStartRegisterPasskeyMutation(client);
  const {
    mutateAsync: registerFinishPasskeyMutateAsync,
    error,
    isError,
  } = useFinishRegisterPasskeyMutation(client);

  useEffect(() => {
    if (isError)
      toast.error('Registration failed', {
        description: (error as ResponseError).details.message,
      });
  }, [isError, error]);

  const [open, setOpen] = useState(false);
  const [passkey, setPasskey] = useState<RegistrationResponseJSON | null>(null);

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      name: '',
    },
  });

  return (
    <>
      <AlertDialog
        open={open}
        onOpenChange={(o) => {
          setOpen(o);
          if (!o)
            setTimeout(() => {
              form.reset();
              setPasskey(null);
            }, 100);
        }}
      >
        <AlertDialogContent className="sm:max-w-[425px]">
          <AlertDialogHeader>
            <AlertDialogTitle>Register passkey</AlertDialogTitle>
          </AlertDialogHeader>

          <Form {...form}>
            <form
              onSubmit={(e) =>
                void form.handleSubmit(async (values) => {
                  await registerFinishPasskeyMutateAsync({
                    ...values,
                    passkey: passkey!,
                  });

                  setOpen(false);
                  setTimeout(() => {
                    form.reset();
                    setPasskey(null);
                  }, 100);
                })(e)
              }
            >
              <div className="space-y-5">
                <FormField
                  control={form.control}
                  name="name"
                  render={({ field }) => (
                    <FormItem className="w-full">
                      <FormLabel>Name</FormLabel>
                      <FormControl>
                        <Input placeholder="Name" {...field} />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                <AlertDialogFooter>
                  <AlertDialogCancel>Cancel</AlertDialogCancel>
                  <AlertDialogAction type="submit">Submit</AlertDialogAction>
                </AlertDialogFooter>
              </div>
            </form>
          </Form>
        </AlertDialogContent>
      </AlertDialog>

      <div className="flex flex-col gap-y-5 p-5">
        {!user?.verified && (
          <Alert
            variant="default"
            className="flex flex-row items-center justify-between"
          >
            <div>
              <AlertTitle>Your email isn&apos;t verified</AlertTitle>
              <AlertDescription>
                Verify your email to unlock all features in the app
              </AlertDescription>
            </div>

            <Button
              variant="outline"
              onClick={() => mutate()}
              disabled={isPending}
            >
              {isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Resend verification
            </Button>
          </Alert>
        )}

        <Card>
          <CardHeader className="flex flex-row justify-between">
            <div className="flex flex-col space-y-1.5">
              <CardTitle>Passkeys</CardTitle>
              <CardDescription>
                Enable login with passkeys for this account
              </CardDescription>
            </div>

            <Button
              variant="secondary"
              onClick={() => {
                void (async () => {
                  const options = await registerStartPasskeyMutateAsync();

                  try {
                    const response = await startRegistration(options.publicKey);
                    setPasskey(response);
                    setOpen(true);
                  } catch (err) {
                    if (
                      (err as WebAuthnError).code ===
                      'ERROR_AUTHENTICATOR_PREVIOUSLY_REGISTERED'
                    )
                      toast.error('Registration failed', {
                        description:
                          'Please use a different authenticator, this one is already registered.',
                      });
                    else throw err;
                  }
                })();
              }}
            >
              <Plus className="mr-2 h-4 w-4" /> Add a passkey
            </Button>
          </CardHeader>

          {passkeys.length > 0 && (
            <CardContent>
              <div className="overflow-hidden rounded-md border">
                <Table>
                  <TableBody>
                    {passkeys.map((passkey) => (
                      <TableRow key={passkey.id}>
                        <TableCell className="py-3 font-medium">
                          <div className="flex flex-row">
                            {passkey.name}
                            <Badge
                              variant="secondary"
                              className="hover:bg-secondary ml-2"
                            >
                              {passkey.lastUsed
                                ? `Last used ${formatDistanceToNow(passkey.lastUsed, { addSuffix: true })}`
                                : 'Never used'}
                            </Badge>
                          </div>
                        </TableCell>
                        <TableCell className="w-[1%] py-3 font-medium">
                          <MultiDialog<'edit'>>
                            {(mdb) => (
                              <>
                                <DropdownMenu>
                                  <DropdownMenuTrigger asChild>
                                    <Button
                                      variant="ghost"
                                      className="h-8 w-8 p-0"
                                    >
                                      <MoreHorizontal className="h-4 w-4 text-white" />
                                    </Button>
                                  </DropdownMenuTrigger>
                                  <DropdownMenuContent
                                    align="end"
                                    className="w-[120px]"
                                  >
                                    <mdb.Trigger value="edit">
                                      <DropdownMenuItem>
                                        <Pencil className="mr-2 h-4 w-4" /> Edit
                                      </DropdownMenuItem>
                                    </mdb.Trigger>

                                    <DropdownMenuItem
                                      onClick={() =>
                                        deletePasskeyMutate(passkey.id)
                                      }
                                    >
                                      <Trash2 className="mr-2 h-4 w-4" /> Delete
                                    </DropdownMenuItem>
                                  </DropdownMenuContent>
                                </DropdownMenu>

                                <mdb.Container value="edit">
                                  {({ open, onOpenChange }) => (
                                    <EditPasskeyDialog
                                      passkey={passkey}
                                      open={open}
                                      onOpenChange={onOpenChange}
                                    />
                                  )}
                                </mdb.Container>
                              </>
                            )}
                          </MultiDialog>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            </CardContent>
          )}
        </Card>

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
                <Button
                  key={provider}
                  asChild
                  variant="outline"
                  className="w-full"
                >
                  <a
                    href={client.accounts.loginUsingOAuth2(provider, {
                      auth: accessToken,
                      to: routerState.location.pathname,
                    })}
                  >
                    {providerIcons[provider]}
                  </a>
                </Button>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </>
  );
}
