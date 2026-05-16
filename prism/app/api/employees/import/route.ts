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
    const { org_id, csv_data } = body

    // Verify user is admin
    const { data: orgMember } = await supabase
      .from('org_members')
      .select('role')
      .eq('user_id', user.id)
      .eq('org_id', org_id)
      .single()

    if (!orgMember || orgMember.role !== 'admin') {
      return NextResponse.json({ error: 'Insufficient permissions' }, { status: 403 })
    }

    // Parse CSV or JSON
    let employees = []
    const trimmedData = csv_data.trim()

    if (trimmedData.startsWith('[')) {
      // JSON format
      try {
        employees = JSON.parse(trimmedData)
      } catch (e) {
        return NextResponse.json({ error: 'Invalid JSON format' }, { status: 400 })
      }
    } else {
      // CSV format
      const lines = trimmedData.split('\n').filter((l) => l.trim())
      if (lines.length < 2) {
        return NextResponse.json({ error: 'No employee data found' }, { status: 400 })
      }

      const headers = lines[0].split(',').map((h) => h.trim())
      const requiredFields = ['employee_id', 'first_name', 'last_name', 'email']

      // Check headers
      if (!requiredFields.every((f) => headers.includes(f))) {
        return NextResponse.json({ error: 'Missing required fields: ' + requiredFields.join(', ') }, { status: 400 })
      }

      for (let i = 1; i < lines.length; i++) {
        const values = lines[i].split(',').map((v) => v.trim())
        const emp: Record<string, string> = {}

        headers.forEach((header, index) => {
          emp[header] = values[index] || ''
        })

        employees.push(emp)
      }
    }

    // Insert employees
    let imported = 0
    const errors: string[] = []

    for (const emp of employees) {
      if (!emp.employee_id || !emp.first_name || !emp.last_name || !emp.email) {
        errors.push(`Skipping invalid record: ${emp.employee_id}`)
        continue
      }

      const { error } = await supabase.from('employees').upsert(
        [
          {
            org_id,
            employee_id: emp.employee_id,
            first_name: emp.first_name,
            last_name: emp.last_name,
            email: emp.email,
            department: emp.department || null,
            designation: emp.designation || null,
            is_active: true,
          },
        ],
        { onConflict: 'org_id,employee_id' }
      )

      if (error) {
        errors.push(`Error importing ${emp.employee_id}: ${error.message}`)
      } else {
        imported++
      }
    }

    return NextResponse.json({ imported, errors, total: employees.length }, { status: 200 })
  } catch (error) {
    console.error('Employees import API error:', error)
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 })
  }
}
