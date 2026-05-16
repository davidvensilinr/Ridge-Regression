'use client'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import { Organization } from '@/lib/types/prism'
import { useState } from 'react'
import { Users, Plus } from 'lucide-react'
import { Textarea } from '@/components/ui/textarea'

interface AdminOrgSettingsProps {
  org?: Organization
  orgId: string
}

export function AdminOrgSettings({ org, orgId }: AdminOrgSettingsProps) {
  const [orgName, setOrgName] = useState(org?.name || '')
  const [isSaving, setIsSaving] = useState(false)
  const [employeeFileContent, setEmployeeFileContent] = useState('')
  const [isImporting, setIsImporting] = useState(false)

  const handleSaveOrgName = async () => {
    setIsSaving(true)
    try {
      const response = await fetch('/api/organization', {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          org_id: orgId,
          name: orgName,
        }),
      })

      if (response.ok) {
        alert('Organization name updated successfully!')
      }
    } finally {
      setIsSaving(false)
    }
  }

  const handleImportEmployees = async () => {
    if (!employeeFileContent.trim()) {
      alert('Please paste employee data')
      return
    }

    setIsImporting(true)
    try {
      const response = await fetch('/api/employees/import', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          org_id: orgId,
          csv_data: employeeFileContent,
        }),
      })

      if (response.ok) {
        const result = await response.json()
        alert(`Successfully imported ${result.imported} employees!`)
        setEmployeeFileContent('')
      }
    } finally {
      setIsImporting(false)
    }
  }

  return (
    <div className="space-y-4">
      {/* Organization Details */}
      <Card>
        <CardHeader>
          <CardTitle>Organization Details</CardTitle>
          <CardDescription>Manage your organization settings</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-4">
            <div>
              <Label>Organization Name</Label>
              <div className="flex gap-2 mt-1">
                <Input
                  value={orgName}
                  onChange={(e) => setOrgName(e.target.value)}
                  placeholder="Enter organization name"
                />
                <Button onClick={handleSaveOrgName} disabled={isSaving}>
                  {isSaving ? 'Saving...' : 'Save'}
                </Button>
              </div>
            </div>
            <div>
              <Label>Organization ID</Label>
              <Input value={orgId} disabled className="font-mono" />
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Employee Bulk Import */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Plus className="h-5 w-5" />
            Bulk Import Employees
          </CardTitle>
          <CardDescription>Import employees from CSV or JSON format</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <Label>Employee Data (CSV or JSON)</Label>
            <Textarea
              placeholder={`Paste employee data here. Format examples:
              
CSV: employee_id,first_name,last_name,email,department,designation
001,John,Doe,john@example.com,Engineering,Senior Developer

JSON:
[{"employee_id":"001","first_name":"John","last_name":"Doe","email":"john@example.com","department":"Engineering","designation":"Senior Developer"}]`}
              value={employeeFileContent}
              onChange={(e) => setEmployeeFileContent(e.target.value)}
              rows={8}
              className="font-mono"
            />
          </div>
          <Button onClick={handleImportEmployees} disabled={isImporting} className="w-full">
            {isImporting ? 'Importing...' : 'Import Employees'}
          </Button>
        </CardContent>
      </Card>

      {/* Team Members */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Users className="h-5 w-5" />
            Team Members
          </CardTitle>
          <CardDescription>Manage organization members and roles</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="text-sm text-muted-foreground">
            <p>Team member management is coming soon. For now, members can be added through their signup and manually assigned roles via the database.</p>
          </div>
        </CardContent>
      </Card>

      {/* Data Management */}
      <Card className="border-orange-200 bg-orange-50/20">
        <CardHeader>
          <CardTitle className="text-orange-900">Data Management</CardTitle>
          <CardDescription>Advanced operations for organization data</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <p className="text-sm font-medium">Reset Organization Data</p>
            <p className="text-xs text-muted-foreground">This will delete all appraisal cycles, scores, and decisions. This action cannot be undone.</p>
            <Button variant="destructive" className="w-full">
              Reset All Data
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
