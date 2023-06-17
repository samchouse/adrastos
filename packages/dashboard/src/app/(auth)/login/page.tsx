import { Card, CardHeader, CardTitle, LoginForm } from '~/components';

const Page: React.FC = () => (
  <main>
    <div className="flex h-screen w-screen items-center justify-center">
      <Card className="mx-6 w-full sm:m-0 sm:w-[500px]">
        <CardHeader>
          <CardTitle>Login</CardTitle>
        </CardHeader>

        <LoginForm />
      </Card>
    </div>
  </main>
);

export default Page;
