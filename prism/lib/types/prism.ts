// PRISM Type Definitions

export type AlgorithmType = 'contribution' | 'engagement' | 'reliability' | 'collaboration' | 'learning'

export type DecisionType = 'hike' | 'promotion' | 'training' | 'performance_improvement' | 'notes'

export type CycleStatus = 'planning' | 'active' | 'completed' | 'archived'

export interface Organization {
  id: string
  name: string
  created_at: string
  updated_at: string
}

export interface Employee {
  id: string
  org_id: string
  user_id?: string
  employee_id: string
  first_name: string
  last_name: string
  email: string
  department?: string
  designation?: string
  manager_id?: string
  joining_date?: string
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface AppraisalCycle {
  id: string
  org_id: string
  name: string
  start_date: string
  end_date: string
  status: CycleStatus
  created_at: string
  updated_at: string
}

export interface CalculatedScore {
  id: string
  cycle_id: string
  employee_id: string
  snapshot_id: string
  algorithm_type: AlgorithmType
  raw_score: number
  normalized_score: number
  percentile?: number
  confidence: number
  components?: Record<string, any>
  created_at: string
  updated_at: string
}

export interface MLPrediction {
  id: string
  cycle_id: string
  employee_id: string
  prediction_type: 'performance_rating' | 'hike_eligibility' | 'retention_risk'
  predicted_value: number
  confidence?: number
  model_version?: string
  explainability?: Record<string, any>
  created_at: string
}

export interface AlgorithmConfig {
  id: string
  org_id: string
  cycle_id?: string
  algorithm_type: AlgorithmType
  weights: Record<string, number>
  is_active: boolean
  created_at: string
  updated_at: string
}

export interface HRDecision {
  id: string
  cycle_id: string
  employee_id: string
  decision_type: DecisionType
  decision_value?: string
  rationale?: string
  created_by: string
  created_at: string
}

export interface EmployeeSnapshot {
  id: string
  cycle_id: string
  employee_id: string
  emp_number: string
  emp_name: string
  emp_email: string
  department?: string
  designation?: string
  manager_name?: string
  created_at: string
}

export interface PrismDashboardStats {
  totalEmployees: number
  activeCycles: number
  pendingDecisions: number
  averagePerformanceRating: number
}

export interface EmployeeScore {
  employee: Employee
  snapshot: EmployeeSnapshot
  scores: CalculatedScore[]
  predictions: MLPrediction[]
  decisions: HRDecision[]
}
