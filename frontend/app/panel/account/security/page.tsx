import { Dashboard } from "@/components/dashboard"
import { Password } from "./password"
import { SSHKeys } from "./ssh-keys"
import { Logout } from "./logout"

export default async function Security() {
  return (
    <Dashboard.Page>
      <Dashboard.Header>
        <Dashboard.Title>Security</Dashboard.Title>
        <Dashboard.Subtext>Keep your account secure.</Dashboard.Subtext>
      </Dashboard.Header>
      <Password />
      <SSHKeys />
      <Logout />
    </Dashboard.Page>
  )
}
