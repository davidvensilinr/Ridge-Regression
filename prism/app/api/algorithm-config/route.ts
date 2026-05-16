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
    const { org_id, weights } = body

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

    // Upsert algorithm configs
    const algorithms = ['contribution', 'engagement', 'reliability', 'collaboration', 'learning']
    const results = []

    for (const algo of algorithms) {
      const { data, error } = await supabase
        .from('algorithm_configs')
        .upsert(
          [
            {
              org_id,
              algorithm_type: algo,
              weights: weights[algo] || {},
              is_active: true,
            },
          ],
          { onConflict: 'org_id,algorithm_type' }
        )
        .select()

      if (error) {
        console.error(`Error updating ${algo} config:`, error)
      } else {
        results.push(data)
      }
    }

    return NextResponse.json({ success: true, data: results }, { status: 200 })
  } catch (error) {
    console.error('Algorithm config API error:', error)
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 })
  }
}
