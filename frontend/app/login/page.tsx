import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { LoginForm } from "@/components/login-form"

export default function LoginPage() {
  return (
    <main className="flex flex-col justify-center flex-1">
      <div className="flex items-center justify-center flex-col gap-4">
        <Card>
          <CardHeader>
            <CardTitle className="pr-48">Welcome back</CardTitle>
            <CardDescription>Please log in to continue.</CardDescription>
          </CardHeader>
          <CardContent>
            <LoginForm />
          </CardContent>
        </Card>
      </div>
    </main>
  )
}
