import { AlgorithmType, CalculatedScore, Employee, EmployeeSnapshot, AlgorithmConfig } from '@/lib/types/prism'

export interface AlgorithmInput {
  employee: Employee
  snapshot: EmployeeSnapshot
  seed: string // for consistent mock generation
}

// Mock data generator - simulates ML algorithm predictions
export function calculateContributionScore(input: AlgorithmInput, weights?: Record<string, number>): CalculatedScore {
  const { employee, snapshot, seed } = input
  const w = weights || { projects: 0.3, impact: 0.3, quality: 0.4 }
  
  // Deterministic pseudo-random based on seed
  const pseudoRandom = generateDeterministicRandom(seed + 'contribution')
  
  const projectScore = 60 + pseudoRandom * 40
  const impactScore = 55 + (pseudoRandom * 0.7) * 45
  const qualityScore = 65 + (pseudoRandom * 0.5) * 35
  
  const rawScore = (projectScore * w.projects + impactScore * w.impact + qualityScore * w.quality)
  const normalizedScore = Math.min(100, Math.max(0, rawScore / 100))
  
  return {
    id: `score_contribution_${employee.id}`,
    cycle_id: '',
    employee_id: employee.id,
    snapshot_id: snapshot.id,
    algorithm_type: 'contribution' as AlgorithmType,
    raw_score: rawScore,
    normalized_score: parseFloat((normalizedScore * 100).toFixed(2)),
    percentile: Math.round(normalizedScore * 100),
    confidence: 0.85,
    components: {
      projects: parseFloat(projectScore.toFixed(2)),
      impact: parseFloat(impactScore.toFixed(2)),
      quality: parseFloat(qualityScore.toFixed(2)),
    },
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }
}

export function calculateEngagementScore(input: AlgorithmInput, weights?: Record<string, number>): CalculatedScore {
  const { employee, snapshot, seed } = input
  const w = weights || { participation: 0.3, initiative: 0.35, feedback: 0.35 }
  
  const pseudoRandom = generateDeterministicRandom(seed + 'engagement')
  
  const participationScore = 50 + pseudoRandom * 50
  const initiativeScore = 45 + (pseudoRandom * 0.8) * 55
  const feedbackScore = 55 + (pseudoRandom * 0.7) * 45
  
  const rawScore = (participationScore * w.participation + initiativeScore * w.initiative + feedbackScore * w.feedback)
  const normalizedScore = Math.min(100, Math.max(0, rawScore / 100))
  
  return {
    id: `score_engagement_${employee.id}`,
    cycle_id: '',
    employee_id: employee.id,
    snapshot_id: snapshot.id,
    algorithm_type: 'engagement' as AlgorithmType,
    raw_score: rawScore,
    normalized_score: parseFloat((normalizedScore * 100).toFixed(2)),
    percentile: Math.round(normalizedScore * 100),
    confidence: 0.82,
    components: {
      participation: parseFloat(participationScore.toFixed(2)),
      initiative: parseFloat(initiativeScore.toFixed(2)),
      feedback: parseFloat(feedbackScore.toFixed(2)),
    },
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }
}

export function calculateReliabilityScore(input: AlgorithmInput, weights?: Record<string, number>): CalculatedScore {
  const { employee, snapshot, seed } = input
  const w = weights || { attendance: 0.25, deadlines: 0.4, consistency: 0.35 }
  
  const pseudoRandom = generateDeterministicRandom(seed + 'reliability')
  
  const attendanceScore = 70 + pseudoRandom * 30
  const deadlinesScore = 60 + (pseudoRandom * 0.9) * 40
  const consistencyScore = 65 + (pseudoRandom * 0.8) * 35
  
  const rawScore = (attendanceScore * w.attendance + deadlinesScore * w.deadlines + consistencyScore * w.consistency)
  const normalizedScore = Math.min(100, Math.max(0, rawScore / 100))
  
  return {
    id: `score_reliability_${employee.id}`,
    cycle_id: '',
    employee_id: employee.id,
    snapshot_id: snapshot.id,
    algorithm_type: 'reliability' as AlgorithmType,
    raw_score: rawScore,
    normalized_score: parseFloat((normalizedScore * 100).toFixed(2)),
    percentile: Math.round(normalizedScore * 100),
    confidence: 0.88,
    components: {
      attendance: parseFloat(attendanceScore.toFixed(2)),
      deadlines: parseFloat(deadlinesScore.toFixed(2)),
      consistency: parseFloat(consistencyScore.toFixed(2)),
    },
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }
}

export function calculateCollaborationScore(input: AlgorithmInput, weights?: Record<string, number>): CalculatedScore {
  const { employee, snapshot, seed } = input
  const w = weights || { teamwork: 0.4, communication: 0.3, support: 0.3 }
  
  const pseudoRandom = generateDeterministicRandom(seed + 'collaboration')
  
  const teamworkScore = 55 + pseudoRandom * 45
  const communicationScore = 60 + (pseudoRandom * 0.8) * 40
  const supportScore = 50 + (pseudoRandom * 0.9) * 50
  
  const rawScore = (teamworkScore * w.teamwork + communicationScore * w.communication + supportScore * w.support)
  const normalizedScore = Math.min(100, Math.max(0, rawScore / 100))
  
  return {
    id: `score_collaboration_${employee.id}`,
    cycle_id: '',
    employee_id: employee.id,
    snapshot_id: snapshot.id,
    algorithm_type: 'collaboration' as AlgorithmType,
    raw_score: rawScore,
    normalized_score: parseFloat((normalizedScore * 100).toFixed(2)),
    percentile: Math.round(normalizedScore * 100),
    confidence: 0.80,
    components: {
      teamwork: parseFloat(teamworkScore.toFixed(2)),
      communication: parseFloat(communicationScore.toFixed(2)),
      support: parseFloat(supportScore.toFixed(2)),
    },
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }
}

export function calculateLearningScore(input: AlgorithmInput, weights?: Record<string, number>): CalculatedScore {
  const { employee, snapshot, seed } = input
  const w = weights || { training: 0.35, skills: 0.35, growth: 0.3 }
  
  const pseudoRandom = generateDeterministicRandom(seed + 'learning')
  
  const trainingScore = 40 + pseudoRandom * 60
  const skillsScore = 50 + (pseudoRandom * 0.7) * 50
  const growthScore = 45 + (pseudoRandom * 0.8) * 55
  
  const rawScore = (trainingScore * w.training + skillsScore * w.skills + growthScore * w.growth)
  const normalizedScore = Math.min(100, Math.max(0, rawScore / 100))
  
  return {
    id: `score_learning_${employee.id}`,
    cycle_id: '',
    employee_id: employee.id,
    snapshot_id: snapshot.id,
    algorithm_type: 'learning' as AlgorithmType,
    raw_score: rawScore,
    normalized_score: parseFloat((normalizedScore * 100).toFixed(2)),
    percentile: Math.round(normalizedScore * 100),
    confidence: 0.78,
    components: {
      training: parseFloat(trainingScore.toFixed(2)),
      skills: parseFloat(skillsScore.toFixed(2)),
      growth: parseFloat(growthScore.toFixed(2)),
    },
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }
}

export function calculateAllScores(input: AlgorithmInput, configs?: Record<AlgorithmType, AlgorithmConfig>): CalculatedScore[] {
  const scores: CalculatedScore[] = []
  
  scores.push(calculateContributionScore(input, configs?.contribution?.weights))
  scores.push(calculateEngagementScore(input, configs?.engagement?.weights))
  scores.push(calculateReliabilityScore(input, configs?.reliability?.weights))
  scores.push(calculateCollaborationScore(input, configs?.collaboration?.weights))
  scores.push(calculateLearningScore(input, configs?.learning?.weights))
  
  return scores
}

// Deterministic PRNG for consistent mock data
function generateDeterministicRandom(seed: string): number {
  let hash = 0
  for (let i = 0; i < seed.length; i++) {
    const char = seed.charCodeAt(i)
    hash = ((hash << 5) - hash) + char
    hash = hash & hash // Convert to 32bit integer
  }
  return Math.abs(hash) % 1000 / 1000
}

// Calculate composite performance rating from all scores
export function calculatePerformanceRating(scores: CalculatedScore[]): number {
  if (scores.length === 0) return 0
  const avgScore = scores.reduce((sum, s) => sum + s.normalized_score, 0) / scores.length
  return Math.round(avgScore)
}

// Determine hike eligibility based on scores and performance
export function calculateHikeEligibility(scores: CalculatedScore[], rating: number): { eligible: boolean; reason: string } {
  const avgScore = scores.reduce((sum, s) => sum + s.normalized_score, 0) / scores.length
  
  if (avgScore >= 75 && rating >= 4) {
    return { eligible: true, reason: 'Excellent performance across all metrics' }
  }
  if (avgScore >= 65 && rating >= 3) {
    return { eligible: true, reason: 'Strong performance metrics' }
  }
  if (avgScore >= 50) {
    return { eligible: false, reason: 'Below average performance - focus on improvement' }
  }
  return { eligible: false, reason: 'Performance below threshold' }
}

// Calculate retention risk based on engagement and learning scores
export function calculateRetentionRisk(scores: CalculatedScore[]): { risk: 'low' | 'medium' | 'high'; score: number } {
  const engagementScore = scores.find(s => s.algorithm_type === 'engagement')?.normalized_score || 0
  const learningScore = scores.find(s => s.algorithm_type === 'learning')?.normalized_score || 0
  
  const riskScore = 100 - (engagementScore + learningScore) / 2
  
  if (riskScore < 30) return { risk: 'low', score: riskScore }
  if (riskScore < 70) return { risk: 'medium', score: riskScore }
  return { risk: 'high', score: riskScore }
}
