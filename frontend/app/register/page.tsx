import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { RegisterForm } from "@/components/register-form"

export default function RegisterPage() {
  return (
    <main className="flex flex-col justify-center flex-1">
      <div className="flex items-center justify-center flex-col gap-4">
        <Card>
          <CardHeader>
            <CardTitle className="pr-64">Welcome</CardTitle>
            <CardDescription>
              Please create an account to continue.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <RegisterForm />
          </CardContent>
        </Card>
      </div>
    </main>
  )
}
