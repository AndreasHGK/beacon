import { Dashboard } from "@/components/dashboard"
import Username from "./username"

export default function Panel() {
  return (
    <Dashboard.Page>
      <Dashboard.Header>
        <Dashboard.Title>Profile</Dashboard.Title>
        <Dashboard.Subtext>Modify general account settings.</Dashboard.Subtext>
      </Dashboard.Header>
      <Username />
    </Dashboard.Page>
  )
}
