import { createClient } from '@/lib/supabase/server'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, LineChart, Line } from 'recharts'
import { Users, TrendingUp, Clock, AlertCircle } from 'lucide-react'

export default async function DashboardPage() {
  const supabase = await createClient()
  const {
    data: { user },
  } = await supabase.auth.getUser()

  // Fetch organization for this user
  const { data: orgMembers } = await supabase
    .from('org_members')
    .select('org_id')
    .eq('user_id', user?.id)
    .single()

  const orgId = orgMembers?.org_id

  // Fetch stats
  let employeeCount = 0
  let cycleCount = 0
  let pendingDecisions = 0

  if (orgId) {
    const [empRes, cycleRes, decisionRes] = await Promise.all([
      supabase.from('employees').select('id').eq('org_id', orgId),
      supabase.from('appraisal_cycles').select('id').eq('org_id', orgId).eq('status', 'active'),
      supabase
        .from('hr_decisions')
        .select('id')
        .eq('created_by', user?.id)
        .gte('created_at', new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString()),
    ])

    employeeCount = empRes.data?.length || 0
    cycleCount = cycleRes.data?.length || 0
    pendingDecisions = decisionRes.data?.length || 0
  }

  // Mock chart data
  const performanceData = [
    { name: 'Contribution', avg: 72, target: 75 },
    { name: 'Engagement', avg: 68, target: 70 },
    { name: 'Reliability', avg: 78, target: 75 },
    { name: 'Collaboration', avg: 65, target: 70 },
    { name: 'Learning', avg: 60, target: 70 },
  ]

  const trendsData = [
    { month: 'Jan', rating: 65 },
    { month: 'Feb', rating: 67 },
    { month: 'Mar', rating: 69 },
    { month: 'Apr', rating: 70 },
    { month: 'May', rating: 72 },
  ]

  return (
    <div className="space-y-8 p-4 sm:p-6 lg:p-8">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold">PRISM Dashboard</h1>
        <p className="text-muted-foreground">AI-powered HR intelligence and talent insights</p>
      </div>

      {/* Stats Grid */}
      <div className="grid gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Employees</CardTitle>
            <Users className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{employeeCount}</div>
            <p className="text-xs text-muted-foreground">Active employees</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Cycles</CardTitle>
            <Clock className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{cycleCount}</div>
            <p className="text-xs text-muted-foreground">Running appraisals</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">HR Decisions</CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{pendingDecisions}</div>
            <p className="text-xs text-muted-foreground">This month</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Avg Rating</CardTitle>
            <AlertCircle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">3.2/5</div>
            <p className="text-xs text-muted-foreground">Organization average</p>
          </CardContent>
        </Card>
      </div>

      {/* Charts */}
      <div className="grid gap-6 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Algorithm Performance</CardTitle>
            <CardDescription>Average scores vs targets by dimension</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <BarChart data={performanceData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="name" />
                <YAxis />
                <Tooltip />
                <Bar dataKey="avg" fill="hsl(var(--primary))" name="Average" />
                <Bar dataKey="target" fill="hsl(var(--muted))" name="Target" />
              </BarChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Performance Trend</CardTitle>
            <CardDescription>Overall rating trend over time</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <LineChart data={trendsData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="month" />
                <YAxis domain={[0, 100]} />
                <Tooltip />
                <Line type="monotone" dataKey="rating" stroke="hsl(var(--primary))" strokeWidth={2} dot={{ fill: 'hsl(var(--primary))' }} />
              </LineChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
