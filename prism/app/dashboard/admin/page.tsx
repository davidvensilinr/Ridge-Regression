import { createClient } from '@/lib/supabase/server'
import { redirect } from 'next/navigation'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Settings, Database, Code2 } from 'lucide-react'
import { AdminAlgorithmConfig } from '@/components/admin/algorithm-config'
import { AdminModelConsole } from '@/components/admin/model-console'
import { AdminOrgSettings } from '@/components/admin/org-settings'

export default async function AdminPage() {
  const supabase = await createClient()
  const {
    data: { user },
  } = await supabase.auth.getUser()

  if (!user) {
    redirect('/auth/login')
  }

  // Check admin role
  const { data: orgMember } = await supabase
    .from('org_members')
    .select('role, org_id')
    .eq('user_id', user.id)
    .single()

  if (orgMember?.role !== 'admin') {
    redirect('/dashboard')
  }

  // Get organization
  const { data: org } = await supabase.from('organizations').select('*').eq('id', orgMember.org_id).single()

  // Get algorithm configs
  const { data: algos } = await supabase
    .from('algorithm_configs')
    .select('*')
    .eq('org_id', orgMember.org_id)

  return (
    <div className="space-y-6 p-4 sm:p-6 lg:p-8">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold">Admin Panel</h1>
        <p className="text-muted-foreground">Organization settings and model configuration</p>
      </div>

      {/* Tabs */}
      <Tabs defaultValue="algorithms" className="w-full">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="algorithms" className="gap-2">
            <Settings className="h-4 w-4 hidden sm:inline" />
            <span className="hidden sm:inline">Algorithms</span>
            <span className="sm:hidden">Config</span>
          </TabsTrigger>
          <TabsTrigger value="console" className="gap-2">
            <Code2 className="h-4 w-4 hidden sm:inline" />
            <span className="hidden sm:inline">Model Console</span>
            <span className="sm:hidden">Console</span>
          </TabsTrigger>
          <TabsTrigger value="organization" className="gap-2">
            <Database className="h-4 w-4 hidden sm:inline" />
            <span className="hidden sm:inline">Organization</span>
            <span className="sm:hidden">Org</span>
          </TabsTrigger>
        </TabsList>

        {/* Algorithm Configuration */}
        <TabsContent value="algorithms" className="space-y-4">
          <AdminAlgorithmConfig orgId={orgMember.org_id} configs={algos || []} />
        </TabsContent>

        {/* Model Console */}
        <TabsContent value="console" className="space-y-4">
          <AdminModelConsole orgId={orgMember.org_id} />
        </TabsContent>

        {/* Organization Settings */}
        <TabsContent value="organization" className="space-y-4">
          <AdminOrgSettings org={org} orgId={orgMember.org_id} />
        </TabsContent>
      </Tabs>
    </div>
  )
}
