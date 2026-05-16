import { createClient } from '@/lib/supabase/server'
import { notFound } from 'next/navigation'
import { Employee, EmployeeSnapshot, CalculatedScore, MLPrediction, HRDecision } from '@/lib/types/prism'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import Link from 'next/link'
import { ArrowLeft } from 'lucide-react'
import { EmployeeIdentity, ScoresSection, PerformanceRating, HikeEligibility, ExplainabilitySection, HRActionsSection, DecisionHistory } from '@/components/employee-report'

interface PageProps {
  params: Promise<{ id: string }>
}

export default async function EmployeeReportPage({ params }: PageProps) {
  const { id } = await params
  const supabase = await createClient()
  const {
    data: { user },
  } = await supabase.auth.getUser()

  // Get user's organization
  const { data: orgMember } = await supabase
    .from('org_members')
    .select('org_id')
    .eq('user_id', user?.id)
    .single()

  if (!orgMember?.org_id) {
    notFound()
  }

  // Get employee
  const { data: employee } = await supabase
    .from('employees')
    .select('*')
    .eq('id', id)
    .eq('org_id', orgMember.org_id)
    .single()

  if (!employee) {
    notFound()
  }

  // Get latest active appraisal cycle
  const { data: cycle } = await supabase
    .from('appraisal_cycles')
    .select('id')
    .eq('org_id', orgMember.org_id)
    .eq('status', 'active')
    .order('created_at', { ascending: false })
    .limit(1)
    .single()

  // Get employee snapshot for this cycle
  let snapshot: EmployeeSnapshot | null = null
  let scores: CalculatedScore[] = []
  let predictions: MLPrediction[] = []
  let decisions: HRDecision[] = []

  if (cycle?.id) {
    const { data: snapshotData } = await supabase
      .from('employee_snapshots')
      .select('*')
      .eq('cycle_id', cycle.id)
      .eq('employee_id', id)
      .single()

    snapshot = snapshotData

    if (snapshot) {
      const [scoresRes, predictionsRes, decisionsRes] = await Promise.all([
        supabase
          .from('calculated_scores')
          .select('*')
          .eq('cycle_id', cycle.id)
          .eq('employee_id', id),
        supabase
          .from('ml_predictions')
          .select('*')
          .eq('cycle_id', cycle.id)
          .eq('employee_id', id),
        supabase
          .from('hr_decisions')
          .select('*')
          .eq('cycle_id', cycle.id)
          .eq('employee_id', id)
          .order('created_at', { ascending: false }),
      ])

      scores = (scoresRes.data as CalculatedScore[]) || []
      predictions = (predictionsRes.data as MLPrediction[]) || []
      decisions = (decisionsRes.data as HRDecision[]) || []
    }
  }

  const empData = employee as Employee
  const performanceRating = scores.length > 0 
    ? Math.round(scores.reduce((sum, s) => sum + s.normalized_score, 0) / scores.length)
    : 0

  const hikeEligibility = scores.length > 0
    ? scores.reduce((sum, s) => sum + s.normalized_score, 0) / scores.length >= 70
    : false

  return (
    <div className="space-y-6 p-4 sm:p-6 lg:p-8">
      {/* Back Button */}
      <Link href="/dashboard/employees">
        <Button variant="ghost" size="sm" className="gap-2">
          <ArrowLeft className="h-4 w-4" />
          Back to Employees
        </Button>
      </Link>

      {/* Main Report Sections */}
      <div className="space-y-6">
        {/* 1. Identity Section */}
        <EmployeeIdentity employee={empData} snapshot={snapshot} />

        {/* 2. Scores Section */}
        {scores.length > 0 && <ScoresSection scores={scores} />}

        {/* 3. Performance Rating */}
        <PerformanceRating rating={performanceRating} />

        {/* 4. Hike Eligibility */}
        <HikeEligibility eligible={hikeEligibility} scores={scores} />

        {/* 5. Explainability */}
        {predictions.length > 0 && <ExplainabilitySection predictions={predictions} scores={scores} />}

        {/* 6. HR Actions */}
        <HRActionsSection employeeId={id} cycleId={cycle?.id} isAdmin={orgMember} user={user} />

        {/* 7. Decision History */}
        {decisions.length > 0 && <DecisionHistory decisions={decisions} />}
      </div>
    </div>
  )
}
