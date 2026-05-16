'use client'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Slider } from '@/components/ui/slider'
import { Badge } from '@/components/ui/badge'
import { AlgorithmConfig } from '@/lib/types/prism'
import { useState } from 'react'

const ALGORITHM_COMPONENTS: Record<string, string[]> = {
  contribution: ['projects', 'impact', 'quality'],
  engagement: ['participation', 'initiative', 'feedback'],
  reliability: ['attendance', 'deadlines', 'consistency'],
  collaboration: ['teamwork', 'communication', 'support'],
  learning: ['training', 'skills', 'growth'],
}

interface AdminAlgorithmConfigProps {
  orgId: string
  configs: AlgorithmConfig[]
}

export function AdminAlgorithmConfig({ orgId, configs }: AdminAlgorithmConfigProps) {
  const [weights, setWeights] = useState<Record<string, Record<string, number>>>(() => {
    const initialWeights: Record<string, Record<string, number>> = {}
    configs.forEach((config) => {
      initialWeights[config.algorithm_type] = config.weights
    })
    return initialWeights
  })

  const [isSaving, setIsSaving] = useState(false)

  const handleWeightChange = (algo: string, component: string, value: number[]) => {
    setWeights((prev) => ({
      ...prev,
      [algo]: {
        ...prev[algo],
        [component]: value[0],
      },
    }))
  }

  const normalizeWeights = (algo: string) => {
    const algoWeights = weights[algo] || {}
    const sum = Object.values(algoWeights).reduce((a, b) => a + b, 0)
    if (sum === 0) return algoWeights

    return Object.entries(algoWeights).reduce(
      (acc, [key, val]) => {
        acc[key] = parseFloat(((val / sum) * 100).toFixed(2))
        return acc
      },
      {} as Record<string, number>
    )
  }

  const handleSaveConfig = async () => {
    setIsSaving(true)
    try {
      const response = await fetch('/api/algorithm-config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          org_id: orgId,
          weights,
        }),
      })

      if (response.ok) {
        alert('Configuration saved successfully!')
      }
    } finally {
      setIsSaving(false)
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Algorithm Weight Configuration</CardTitle>
        <CardDescription>Adjust weights for each algorithm component. Weights will be normalized to 100%.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-8">
        {Object.entries(ALGORITHM_COMPONENTS).map(([algo, components]) => {
          const normalizedWeights = normalizeWeights(algo)

          return (
            <div key={algo} className="space-y-4 border-b pb-6 last:border-b-0">
              <h3 className="font-semibold capitalize text-lg">{algo}</h3>

              <div className="space-y-4">
                {components.map((component) => {
                  const currentWeight = weights[algo]?.[component] || 33
                  const normalizedWeight = normalizedWeights[component] || 0

                  return (
                    <div key={component} className="space-y-2">
                      <div className="flex items-center justify-between">
                        <label className="text-sm font-medium capitalize">{component}</label>
                        <div className="flex gap-2 items-center">
                          <Badge variant="secondary">{currentWeight.toFixed(1)}</Badge>
                          <span className="text-xs text-muted-foreground">{normalizedWeight.toFixed(1)}% (norm.)</span>
                        </div>
                      </div>
                      <Slider
                        min={0}
                        max={100}
                        step={1}
                        value={[currentWeight]}
                        onValueChange={(val) => handleWeightChange(algo, component, val)}
                        className="w-full"
                      />
                    </div>
                  )
                })}
              </div>

              <div className="bg-muted/50 p-3 rounded text-sm">
                <p className="font-medium mb-1">Normalized Distribution:</p>
                <div className="flex gap-2 flex-wrap">
                  {components.map((comp) => (
                    <Badge key={comp} variant="outline">
                      {comp}: {normalizeWeights(algo)[comp]?.toFixed(1)}%
                    </Badge>
                  ))}
                </div>
              </div>
            </div>
          )
        })}

        <Button onClick={handleSaveConfig} disabled={isSaving} className="w-full mt-4">
          {isSaving ? 'Saving...' : 'Save Configuration'}
        </Button>
      </CardContent>
    </Card>
  )
}
