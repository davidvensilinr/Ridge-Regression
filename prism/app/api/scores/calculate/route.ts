import { createClient } from '@/lib/supabase/server'
import { NextRequest, NextResponse } from 'next/server'
import {
  calculateAllScores,
  calculatePerformanceRating,
  calculateHikeEligibility,
  calculateRetentionRisk,
} from '@/lib/prism/algorithms'
import { Employee, EmployeeSnapshot, AlgorithmConfig } from '@/lib/types/prism'

export async function POST(request: NextRequest) {
  try {
    const supabase = await createClient()
    const {
      data: { user },
    } = await supabase.auth.getUser()

    if (!user) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 })
    }

    const body = await request.json()
    const { cycle_id, employee_ids } = body

    // Get cycle and org
    const { data: cycle, error: cycleError } = await supabase
      .from('appraisal_cycles')
      .select('*')
      .eq('id', cycle_id)
      .single()

    if (cycleError || !cycle) {
      return NextResponse.json({ error: 'Cycle not found' }, { status: 404 })
    }

    // Verify user belongs to org
    const { data: orgMember } = await supabase
      .from('org_members')
      .select('role')
      .eq('user_id', user.id)
      .eq('org_id', cycle.org_id)
      .single()

    if (!orgMember || !['admin', 'hr'].includes(orgMember.role)) {
      return NextResponse.json({ error: 'Insufficient permissions' }, { status: 403 })
    }

    // Get algorithm configs
    const { data: configs } = await supabase
      .from('algorithm_configs')
      .select('*')
      .eq('org_id', cycle.org_id)
      .eq('is_active', true)

    const configMap: Record<string, AlgorithmConfig> = {}
    configs?.forEach((c) => {
      configMap[c.algorithm_type] = c
    })

    let calculatedCount = 0
    const results = []

    // Calculate scores for each employee
    for (const empId of employee_ids) {
      // Get employee
      const { data: employee } = await supabase.from('employees').select('*').eq('id', empId).single()

      if (!employee) continue

      // Get or create snapshot
      let { data: snapshot } = await supabase
        .from('employee_snapshots')
        .select('*')
        .eq('cycle_id', cycle_id)
        .eq('employee_id', empId)
        .single()

      if (!snapshot) {
        const { data: newSnapshot } = await supabase
          .from('employee_snapshots')
          .insert([
            {
              cycle_id,
              employee_id: empId,
              emp_number: employee.employee_id,
              emp_name: `${employee.first_name} ${employee.last_name}`,
              emp_email: employee.email,
              department: employee.department,
              designation: employee.designation,
            },
          ])
          .select()
          .single()

        snapshot = newSnapshot
      }

      if (!snapshot) continue

      // Calculate scores
      const scores = calculateAllScores(
        {
          employee: employee as Employee,
          snapshot: snapshot as EmployeeSnapshot,
          seed: empId,
        },
        configMap
      )

      // Update snapshot_id on scores
      scores.forEach((s) => {
        s.snapshot_id = snapshot!.id
        s.cycle_id = cycle_id
      })

      // Upsert scores
      const { error: scoresError } = await supabase.from('calculated_scores').upsert(scores, {
        onConflict: 'cycle_id,employee_id,algorithm_type',
      })

      if (!scoresError) {
        calculatedCount++

        // Calculate and store predictions
        const rating = calculatePerformanceRating(scores)
        const hikeElig = calculateHikeEligibility(scores, rating)
        const retentionRisk = calculateRetentionRisk(scores)

        // Insert predictions
        await supabase.from('ml_predictions').upsert(
          [
            {
              cycle_id,
              employee_id: empId,
              prediction_type: 'performance_rating',
              predicted_value: rating,
              confidence: 0.85,
              explainability: { method: 'ensemble', factors: ['contribution', 'engagement', 'reliability'] },
            },
            {
              cycle_id,
              employee_id: empId,
              prediction_type: 'hike_eligibility',
              predicted_value: hikeElig.eligible ? 1 : 0,
              confidence: 0.88,
              explainability: { reason: hikeElig.reason, threshold: 70 },
            },
            {
              cycle_id,
              employee_id: empId,
              prediction_type: 'retention_risk',
              predicted_value: retentionRisk.score,
              confidence: 0.80,
              explainability: { risk_level: retentionRisk.risk, factors: ['engagement', 'learning'] },
            },
          ],
          { onConflict: 'cycle_id,employee_id,prediction_type' }
        )

        results.push({
          employee_id: empId,
          scores: scores.length,
          rating,
          hikeEligible: hikeElig.eligible,
        })
      }
    }

    return NextResponse.json(
      {
        success: true,
        calculated: calculatedCount,
        total: employee_ids.length,
        results,
      },
      { status: 200 }
    )
  } catch (error) {
    console.error('Scores calculation API error:', error)
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 })
  }
}
