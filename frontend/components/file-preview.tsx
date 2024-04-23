import { File } from "lucide-react"
import { Card, CardContent } from "@/components/ui/card"
import { Separator } from "@/components/ui/separator"

export function FilePreview(params: { name: string; size: number }) {
  return (
    <Card className="pt-6 md:pr-1">
      <CardContent>
        <div className="flex flex-col md:flex-row gap-2">
          <File className="size-60" strokeWidth={1} />
          <Separator orientation="vertical" />
          <div className="flex flex-col gap-2 justify-center md:pr-6 text-center md:text-left">
            <p className="text-3xl font-semibold">{params.name}</p>
            <p className="text-lg text-muted-foreground">{params.size} B</p>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
