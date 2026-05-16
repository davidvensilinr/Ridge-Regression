import { createClient } from '@/lib/supabase/server'
import { NextRequest, NextResponse } from 'next/server'

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
    const { cycle_id, employee_id, decision_type, decision_value, rationale } = body

    // Verify user belongs to the organization
    const { data: employee } = await supabase.from('employees').select('org_id').eq('id', employee_id).single()

    if (!employee) {
      return NextResponse.json({ error: 'Employee not found' }, { status: 404 })
    }

    const { data: orgMember } = await supabase
      .from('org_members')
      .select('role')
      .eq('user_id', user.id)
      .eq('org_id', employee.org_id)
      .single()

    if (!orgMember || !['admin', 'hr'].includes(orgMember.role)) {
      return NextResponse.json({ error: 'Insufficient permissions' }, { status: 403 })
    }

    // Insert the decision
    const { data, error } = await supabase.from('hr_decisions').insert([
      {
        cycle_id,
        employee_id,
        decision_type,
        decision_value,
        rationale,
        created_by: user.id,
      },
    ])

    if (error) {
      return NextResponse.json({ error: error.message }, { status: 400 })
    }

    return NextResponse.json(data, { status: 201 })
  } catch (error) {
    console.error('HR Decision API error:', error)
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 })
  }
}
