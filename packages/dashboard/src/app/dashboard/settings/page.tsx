import { OAuth2Card, SmtpCard } from './_components';

const Page: React.FC = () => (
  <div className="flex flex-col gap-y-6">
    <SmtpCard />
    <OAuth2Card />
  </div>
);

export default Page;
