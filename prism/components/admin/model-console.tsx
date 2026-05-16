'use client'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { useState, useEffect } from 'react'
import { AlertCircle, CheckCircle, Clock } from 'lucide-react'

interface LogEntry {
  id: string
  log_type: 'prediction' | 'error' | 'performance' | 'debug'
  message: string
  metadata?: Record<string, any>
  created_at: string
}

interface AdminModelConsoleProps {
  orgId: string
}

export function AdminModelConsole({ orgId }: AdminModelConsoleProps) {
  const [logs, setLogs] = useState<LogEntry[]>([])
  const [filter, setFilter] = useState<string>('all')
  const [isLoading, setIsLoading] = useState(false)

  useEffect(() => {
    loadLogs()
  }, [orgId, filter])

  const loadLogs = async () => {
    setIsLoading(true)
    try {
      const params = new URLSearchParams({
        org_id: orgId,
        ...(filter !== 'all' && { type: filter }),
      })
      const response = await fetch(`/api/model-logs?${params}`)
      if (response.ok) {
        const data = await response.json()
        setLogs(data)
      }
    } finally {
      setIsLoading(false)
    }
  }

  const clearLogs = async () => {
    if (!confirm('Clear all logs? This action cannot be undone.')) return

    try {
      const response = await fetch('/api/model-logs', {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ org_id: orgId }),
      })
      if (response.ok) {
        setLogs([])
      }
    } catch (error) {
      console.error('Failed to clear logs:', error)
    }
  }

  const getLogIcon = (type: string) => {
    switch (type) {
      case 'error':
        return <AlertCircle className="h-4 w-4 text-red-500" />
      case 'prediction':
        return <CheckCircle className="h-4 w-4 text-green-500" />
      case 'performance':
        return <Clock className="h-4 w-4 text-blue-500" />
      default:
        return <Clock className="h-4 w-4 text-gray-500" />
    }
  }

  const getLogColor = (type: string) => {
    switch (type) {
      case 'error':
        return 'bg-red-50 border-red-200'
      case 'prediction':
        return 'bg-green-50 border-green-200'
      case 'performance':
        return 'bg-blue-50 border-blue-200'
      default:
        return 'bg-gray-50 border-gray-200'
    }
  }

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle>Model Console & Logs</CardTitle>
          <CardDescription>Monitor model predictions, errors, and performance metrics</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-2 items-center flex-wrap">
            {['all', 'prediction', 'error', 'performance', 'debug'].map((f) => (
              <Button
                key={f}
                size="sm"
                variant={filter === f ? 'default' : 'outline'}
                onClick={() => setFilter(f)}
                className="capitalize"
              >
                {f}
              </Button>
            ))}
            <Button size="sm" variant="destructive" onClick={clearLogs}>
              Clear All
            </Button>
          </div>

          {isLoading ? (
            <div className="text-center py-8 text-muted-foreground">Loading logs...</div>
          ) : logs.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">No logs found</div>
          ) : (
            <div className="space-y-2 max-h-96 overflow-y-auto border rounded-lg p-4">
              {logs.map((log) => (
                <div key={log.id} className={`border p-3 rounded text-sm ${getLogColor(log.log_type)}`}>
                  <div className="flex items-start gap-2">
                    {getLogIcon(log.log_type)}
                    <div className="flex-1 min-w-0">
                      <p className="font-medium">
                        <Badge className="mr-2 capitalize" variant="secondary">
                          {log.log_type}
                        </Badge>
                        {new Date(log.created_at).toLocaleTimeString()}
                      </p>
                      <p className="text-xs mt-1 break-words">{log.message}</p>
                      {log.metadata && (
                        <pre className="text-xs mt-2 bg-black/5 p-2 rounded overflow-x-auto">
                          {JSON.stringify(log.metadata, null, 2)}
                        </pre>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Performance Statistics</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 grid-cols-2 sm:grid-cols-4">
            <div className="border rounded p-3">
              <p className="text-xs text-muted-foreground">Total Predictions</p>
              <p className="text-2xl font-bold">{logs.filter((l) => l.log_type === 'prediction').length}</p>
            </div>
            <div className="border rounded p-3">
              <p className="text-xs text-muted-foreground">Errors</p>
              <p className="text-2xl font-bold text-red-600">{logs.filter((l) => l.log_type === 'error').length}</p>
            </div>
            <div className="border rounded p-3">
              <p className="text-xs text-muted-foreground">Performance Logs</p>
              <p className="text-2xl font-bold">{logs.filter((l) => l.log_type === 'performance').length}</p>
            </div>
            <div className="border rounded p-3">
              <p className="text-xs text-muted-foreground">Success Rate</p>
              <p className="text-2xl font-bold">
                {logs.length > 0
                  ? Math.round(
                      (logs.filter((l) => l.log_type !== 'error').length / logs.length) * 100
                    )
                  : 0}
                %
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
