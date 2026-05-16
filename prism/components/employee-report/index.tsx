'use client'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, LineChart, Line, RadarChart, PolarGrid, PolarAngleAxis, PolarRadiusAxis, Radar } from 'recharts'
import { Employee, EmployeeSnapshot, CalculatedScore, MLPrediction, HRDecision } from '@/lib/types/prism'
import { FileText, CheckCircle, AlertCircle, User, Mail, Briefcase } from 'lucide-react'
import { useState } from 'react'
import { Textarea } from '@/components/ui/textarea'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

// 1. Identity Section
export function EmployeeIdentity({ employee, snapshot }: { employee: Employee; snapshot: EmployeeSnapshot | null }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <User className="h-5 w-5" />
          Employee Identity
        </CardTitle>
        <CardDescription>Personal and organizational information</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          <div>
            <p className="text-sm text-muted-foreground">Name</p>
            <p className="text-lg font-semibold">
              {employee.first_name} {employee.last_name}
            </p>
          </div>
          <div>
            <p className="text-sm text-muted-foreground">Employee ID</p>
            <p className="text-lg font-semibold">{employee.employee_id}</p>
          </div>
          <div>
            <p className="text-sm text-muted-foreground">Email</p>
            <p className="text-sm font-mono">{employee.email}</p>
          </div>
          <div>
            <p className="text-sm text-muted-foreground">Department</p>
            <p className="text-lg font-semibold">{employee.department || 'N/A'}</p>
          </div>
          <div>
            <p className="text-sm text-muted-foreground">Designation</p>
            <p className="text-lg font-semibold">{employee.designation || 'N/A'}</p>
          </div>
          <div>
            <p className="text-sm text-muted-foreground">Joining Date</p>
            <p className="text-lg font-semibold">{employee.joining_date ? new Date(employee.joining_date).toLocaleDateString() : 'N/A'}</p>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}

// 2. Scores Section
export function ScoresSection({ scores }: { scores: CalculatedScore[] }) {
  const radarData = scores.map((s) => ({
    name: s.algorithm_type.charAt(0).toUpperCase() + s.algorithm_type.slice(1),
    score: s.normalized_score,
    fullMark: 100,
  }))

  const barData = scores.map((s) => ({
    name: s.algorithm_type.charAt(0).toUpperCase() + s.algorithm_type.slice(1),
    score: s.normalized_score,
    confidence: s.confidence ? Math.round(s.confidence * 100) : 85,
  }))

  return (
    <div className="grid gap-6 lg:grid-cols-2">
      <Card>
        <CardHeader>
          <CardTitle>Algorithm Scores (Radar View)</CardTitle>
          <CardDescription>5-dimension talent assessment</CardDescription>
        </CardHeader>
        <CardContent>
          <ResponsiveContainer width="100%" height={300}>
            <RadarChart data={radarData}>
              <PolarGrid />
              <PolarAngleAxis dataKey="name" />
              <PolarRadiusAxis angle={90} domain={[0, 100]} />
              <Radar name="Score" dataKey="score" stroke="hsl(var(--primary))" fill="hsl(var(--primary))" fillOpacity={0.6} />
              <Tooltip />
            </RadarChart>
          </ResponsiveContainer>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Scores with Confidence</CardTitle>
          <CardDescription>Normalized scores and model confidence</CardDescription>
        </CardHeader>
        <CardContent>
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={barData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="name" />
              <YAxis domain={[0, 100]} />
              <Tooltip />
              <Bar dataKey="score" fill="hsl(var(--primary))" name="Score" />
              <Bar dataKey="confidence" fill="hsl(var(--muted))" name="Confidence %" />
            </BarChart>
          </ResponsiveContainer>
        </CardContent>
      </Card>

      {/* Detailed Scores Table */}
      <Card className="lg:col-span-2">
        <CardHeader>
          <CardTitle>Detailed Algorithm Breakdown</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b">
                  <th className="text-left py-2 px-2">Algorithm</th>
                  <th className="text-right py-2 px-2">Raw Score</th>
                  <th className="text-right py-2 px-2">Normalized</th>
                  <th className="text-right py-2 px-2">Percentile</th>
                  <th className="text-right py-2 px-2">Confidence</th>
                </tr>
              </thead>
              <tbody>
                {scores.map((score) => (
                  <tr key={score.id} className="border-b hover:bg-muted/50">
                    <td className="py-3 px-2 font-medium capitalize">{score.algorithm_type}</td>
                    <td className="text-right py-3 px-2">{score.raw_score.toFixed(2)}</td>
                    <td className="text-right py-3 px-2">
                      <Badge variant={score.normalized_score >= 75 ? 'default' : score.normalized_score >= 50 ? 'secondary' : 'destructive'}>
                        {score.normalized_score.toFixed(1)}/100
                      </Badge>
                    </td>
                    <td className="text-right py-3 px-2">{score.percentile ? `${score.percentile}%` : 'N/A'}</td>
                    <td className="text-right py-3 px-2">{(score.confidence * 100).toFixed(0)}%</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

// 3. Performance Rating
export function PerformanceRating({ rating }: { rating: number }) {
  const getRatingLabel = (r: number) => {
    if (r >= 90) return { label: 'Exceptional', color: 'bg-green-500' }
    if (r >= 75) return { label: 'Excellent', color: 'bg-blue-500' }
    if (r >= 60) return { label: 'Good', color: 'bg-yellow-500' }
    if (r >= 50) return { label: 'Satisfactory', color: 'bg-orange-500' }
    return { label: 'Needs Improvement', color: 'bg-red-500' }
  }

  const ratingInfo = getRatingLabel(rating)

  return (
    <Card>
      <CardHeader>
        <CardTitle>Performance Rating</CardTitle>
        <CardDescription>Aggregate assessment across all dimensions</CardDescription>
      </CardHeader>
      <CardContent className="flex items-center justify-center py-8">
        <div className="flex flex-col items-center gap-4">
          <div className={`${ratingInfo.color} rounded-full w-32 h-32 flex items-center justify-center`}>
            <div className="text-center">
              <p className="text-5xl font-bold text-white">{rating}</p>
              <p className="text-sm text-white/90">/100</p>
            </div>
          </div>
          <div className="text-center">
            <p className="text-lg font-semibold">{ratingInfo.label}</p>
            <p className="text-sm text-muted-foreground">Overall performance score</p>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}

// 4. Hike Eligibility
export function HikeEligibility({ eligible, scores }: { eligible: boolean; scores: CalculatedScore[] }) {
  const avgScore = scores.length > 0 ? scores.reduce((sum, s) => sum + s.normalized_score, 0) / scores.length : 0

  return (
    <Card className={eligible ? 'border-green-500/50 bg-green-50/20' : 'border-orange-500/50 bg-orange-50/20'}>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          {eligible ? <CheckCircle className="h-5 w-5 text-green-600" /> : <AlertCircle className="h-5 w-5 text-orange-600" />}
          Hike Eligibility Assessment
        </CardTitle>
        <CardDescription>Recommendation based on performance metrics</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div>
          <Badge className={eligible ? 'bg-green-600 hover:bg-green-700' : 'bg-orange-600 hover:bg-orange-700'}>
            {eligible ? 'Eligible' : 'Not Eligible'}
          </Badge>
        </div>
        <div className="space-y-2">
          <p className="font-semibold">Assessment Criteria:</p>
          <ul className="space-y-1 text-sm">
            <li className={eligible ? 'text-green-700' : 'text-orange-700'}>
              {eligible ? '✓' : '✗'} Average Score: {avgScore.toFixed(1)}/100 {eligible ? '(≥70)' : '(<70)'}
            </li>
            <li className="text-muted-foreground">
              • Consistency across all dimensions
            </li>
            <li className="text-muted-foreground">
              • Engagement and learning potential
            </li>
            <li className="text-muted-foreground">
              • Reliability and collaboration scores
            </li>
          </ul>
        </div>
        {eligible && (
          <div className="pt-2">
            <p className="text-sm font-semibold text-green-700">Recommendation: Eligible for salary increment review</p>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

// 5. Explainability Section
export function ExplainabilitySection({ predictions, scores }: { predictions: MLPrediction[]; scores: CalculatedScore[] }) {
  const performanceRating = scores.length > 0
    ? Math.round(scores.reduce((sum, s) => sum + s.normalized_score, 0) / scores.length)
    : 0

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <FileText className="h-5 w-5" />
          Model Explainability
        </CardTitle>
        <CardDescription>How algorithms arrived at their recommendations</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {predictions.map((pred) => (
          <div key={pred.id} className="border rounded-lg p-4 space-y-3">
            <div className="flex items-start justify-between">
              <div>
                <p className="font-semibold capitalize">{pred.prediction_type.replace(/_/g, ' ')}</p>
                <p className="text-sm text-muted-foreground">Confidence: {((pred.confidence || 0) * 100).toFixed(0)}%</p>
              </div>
              <p className="text-2xl font-bold">{Math.round(pred.predicted_value)}</p>
            </div>

            {pred.explainability && (
              <div className="bg-muted/50 rounded p-3 text-sm space-y-2">
                <p className="font-medium">Key Factors:</p>
                <ul className="list-disc list-inside space-y-1">
                  {Object.entries(pred.explainability as Record<string, any>).map(([key, value]) => (
                    <li key={key}>
                      <strong>{key}:</strong> {typeof value === 'number' ? value.toFixed(2) : String(value)}
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        ))}
      </CardContent>
    </Card>
  )
}

// 6. HR Actions Section
export function HRActionsSection({ employeeId, cycleId, isAdmin, user }: any) {
  const [decisionType, setDecisionType] = useState('')
  const [rationale, setRationale] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)

  const handleSubmitDecision = async () => {
    if (!decisionType || !cycleId) return

    setIsSubmitting(true)
    try {
      const response = await fetch('/api/hr-decisions', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          cycle_id: cycleId,
          employee_id: employeeId,
          decision_type: decisionType,
          rationale: rationale || null,
          created_by: user?.id,
        }),
      })

      if (response.ok) {
        setDecisionType('')
        setRationale('')
        window.location.reload()
      }
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>HR Actions & Decisions</CardTitle>
        <CardDescription>Record HR decisions and actions for this employee</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid gap-4 sm:grid-cols-2">
          <div>
            <label className="text-sm font-medium">Decision Type</label>
            <Select value={decisionType} onValueChange={setDecisionType}>
              <SelectTrigger>
                <SelectValue placeholder="Select action..." />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="hike">Salary Hike</SelectItem>
                <SelectItem value="promotion">Promotion</SelectItem>
                <SelectItem value="training">Training Program</SelectItem>
                <SelectItem value="performance_improvement">Performance Improvement</SelectItem>
                <SelectItem value="notes">General Notes</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <div>
          <label className="text-sm font-medium">Rationale</label>
          <Textarea
            placeholder="Provide reasoning for this decision..."
            value={rationale}
            onChange={(e) => setRationale(e.target.value)}
            rows={4}
          />
        </div>

        <Button onClick={handleSubmitDecision} disabled={!decisionType || isSubmitting} className="w-full">
          {isSubmitting ? 'Recording...' : 'Record Decision'}
        </Button>
      </CardContent>
    </Card>
  )
}

// 7. Decision History
export function DecisionHistory({ decisions }: { decisions: HRDecision[] }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Decision & Action History</CardTitle>
        <CardDescription>Audit trail of all HR decisions for this employee</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {decisions.map((decision) => (
            <div key={decision.id} className="border-l-4 border-primary pl-4 py-2">
              <div className="flex items-start justify-between">
                <div>
                  <p className="font-semibold capitalize">{decision.decision_type.replace(/_/g, ' ')}</p>
                  {decision.decision_value && <p className="text-sm text-muted-foreground">Value: {decision.decision_value}</p>}
                </div>
                <p className="text-xs text-muted-foreground">{new Date(decision.created_at).toLocaleDateString()}</p>
              </div>
              {decision.rationale && <p className="text-sm mt-2">{decision.rationale}</p>}
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  )
}
