import { Dashboard } from "@/components/dashboard"
import { LogoutButton } from "@/components/logout"

export function Logout() {
  return (
    <Dashboard.Section>
      <Dashboard.Subtitle>Log Out</Dashboard.Subtitle>
      <Dashboard.Subtext>
        Removes this session, logging you out. You can always come back!
      </Dashboard.Subtext>
      <Dashboard.SectionContent>
        <LogoutButton />
      </Dashboard.SectionContent>
    </Dashboard.Section>
  )
}
