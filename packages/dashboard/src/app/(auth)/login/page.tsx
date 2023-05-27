import {
  SiDiscord,
  SiFacebook,
  SiGithub,
  SiGoogle,
  SiTwitter
} from '@icons-pack/react-simple-icons';
import Link from 'next/link';
import {
  Button,
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
  Input,
  Label
} from '~/components';

const Page = () => (
  <main>
    <div className="flex h-screen w-screen items-center justify-center">
      <Card className="w-[400px]">
        <CardHeader>
          <CardTitle>Login</CardTitle>
        </CardHeader>
        <CardContent>
          <form>
            <div className="flex flex-col gap-1">
              <div>
                <Label htmlFor="email">Email</Label>
                <Input id="email" type="email" placeholder="Email" />
              </div>
              <div>
                <Label htmlFor="password">Password</Label>
                <Input id="password" type="password" placeholder="Password" />
              </div>
            </div>
          </form>
        </CardContent>
        <CardFooter>
          <div className="w-full">
            <Button className="w-full">Login</Button>

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
              <Button variant="outline" className="w-full">
                <SiGoogle className="h-4 w-4" />
              </Button>
              <Button variant="outline" className=" w-full">
                <SiFacebook className="h-4 w-4" />
              </Button>
              <Button variant="outline" className="w-full">
                <SiGithub className="h-4 w-4" />
              </Button>
              <Button variant="outline" className="w-full">
                <SiTwitter className="h-4 w-4" />
              </Button>
              <Button variant="outline" className="w-full">
                <SiDiscord className="h-4 w-4" />
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
      </Card>
    </div>
  </main>
);

export default Page;
