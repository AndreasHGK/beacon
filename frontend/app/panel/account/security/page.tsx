import { SSHKeys } from "./ssh-keys"

export default async function Security() {
  return (
    <main className="flex flex-col justify-center flex-1 gap-8">
      <div className="flex flex-col gap-2">
        <h1 className="font-bold text-4xl">Security</h1>
        <p className="text-lg text-muted-foreground">
          Keep your account secure.
        </p>
      </div>
      <SSHKeys />
    </main>
  )
}
