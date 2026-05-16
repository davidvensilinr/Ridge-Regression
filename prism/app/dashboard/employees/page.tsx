import { createClient } from '@/lib/supabase/server'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import Link from 'next/link'
import { Employee } from '@/lib/types/prism'
import { ChevronRight } from 'lucide-react'

export default async function EmployeesPage() {
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

  let employees: Employee[] = []

  if (orgMember?.org_id) {
    const { data } = await supabase
      .from('employees')
      .select('*')
      .eq('org_id', orgMember.org_id)
      .eq('is_active', true)
      .order('first_name')

    employees = (data as Employee[]) || []
  }

  return (
    <div className="space-y-6 p-4 sm:p-6 lg:p-8">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold">Employees</h1>
        <p className="text-muted-foreground">Manage and view employee profiles and performance</p>
      </div>

      {/* Search and Filter */}
      <div className="flex gap-4">
        <Input placeholder="Search employees..." className="max-w-sm" />
        <Button>Filter</Button>
      </div>

      {/* Employees Grid */}
      <div className="grid gap-4 grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
        {employees.length === 0 ? (
          <div className="col-span-full">
            <Card>
              <CardContent className="pt-6">
                <p className="text-center text-muted-foreground">No employees found. Add your first employee to get started.</p>
              </CardContent>
            </Card>
          </div>
        ) : (
          employees.map((emp) => (
            <Link key={emp.id} href={`/dashboard/employees/${emp.id}`}>
              <Card className="cursor-pointer hover:border-foreground/50 hover:shadow-md transition-all h-full">
                <CardHeader>
                  <div className="flex items-start justify-between">
                    <div>
                      <CardTitle>
                        {emp.first_name} {emp.last_name}
                      </CardTitle>
                      <CardDescription>{emp.designation || 'N/A'}</CardDescription>
                    </div>
                    <ChevronRight className="h-5 w-5 text-muted-foreground" />
                  </div>
                </CardHeader>
                <CardContent>
                  <div className="space-y-2 text-sm">
                    <div>
                      <p className="text-muted-foreground">ID</p>
                      <p className="font-medium">{emp.employee_id}</p>
                    </div>
                    <div>
                      <p className="text-muted-foreground">Department</p>
                      <p className="font-medium">{emp.department || 'N/A'}</p>
                    </div>
                    <div>
                      <p className="text-muted-foreground">Email</p>
                      <p className="font-medium text-xs">{emp.email}</p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </Link>
          ))
        )}
      </div>
    </div>
  )
}
