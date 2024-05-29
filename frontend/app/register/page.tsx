import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { RegisterForm } from "@/components/register-form"
import { getConfig } from "@/lib/config"

export default async function RegisterPage() {
  const config = await getConfig()
  return (
    <main className="flex flex-col justify-center flex-1">
      <div className="flex items-center justify-center flex-col gap-4">
        <Card className="max-w-lg">
          <CardHeader>
            <CardTitle className="pr-64">Welcome</CardTitle>
            <CardDescription>
              Please create an account to continue.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <RegisterForm require_invite_code={!config.disable_invite_codes} />
          </CardContent>
        </Card>
      </div>
    </main>
  )
}
