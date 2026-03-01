import { useCallback, useEffect, useRef, useState } from 'react'
import { Link } from 'react-router-dom'
import { api } from '@/api/client'
import { useSSE } from '@/hooks/useSSE'
import { CommandBlock } from '@/components/shared/CommandBlock'
import { CheckCircle, Circle, Target, GitBranch, Rocket, Loader2, Users } from 'lucide-react'
import type { AgentDefinition, DocsSseEvent } from '@/lib/types'

interface StepState {
  descriptionDone: boolean
  visionDone: boolean
  architectureDone: boolean
  teamDone: boolean
}

export function SetupPage() {
  const [projectName, setProjectName] = useState('')
  const [description, setDescription] = useState('')
  const [saving, setSaving] = useState(false)
  const [saveError, setSaveError] = useState<string | null>(null)

  const [visionContent, setVisionContent] = useState('')
  const [savingVision, setSavingVision] = useState(false)
  const [aligningVision, setAligningVision] = useState(false)

  const [architectureContent, setArchitectureContent] = useState('')
  const [savingArchitecture, setSavingArchitecture] = useState(false)
  const [aligningArchitecture, setAligningArchitecture] = useState(false)

  const [recruitingTeam, setRecruitingTeam] = useState(false)
  const [agents, setAgents] = useState<AgentDefinition[]>([])

  const [done, setDone] = useState<StepState>({
    descriptionDone: false,
    visionDone: false,
    architectureDone: false,
    teamDone: false,
  })
  const [currentStep, setCurrentStep] = useState(1)
  const initialized = useRef(false)

  // Load initial state on mount
  useEffect(() => {
    if (initialized.current) return
    initialized.current = true

    Promise.all([
      api.getConfig().catch(() => null),
      api.getVision().catch(() => null),
      api.getArchitecture().catch(() => null),
      api.getProjectAgents().catch(() => [] as AgentDefinition[]),
    ]).then(([config, vision, arch, existingAgents]) => {
      let descDone = false
      let visDone = false
      let archDone = false
      const teamDoneNow = existingAgents.length > 0

      if (config) {
        setProjectName(config.project.name)
        if (config.project.description) {
          setDescription(config.project.description)
          descDone = true
        }
      }
      if (vision?.exists && vision.content) {
        setVisionContent(vision.content)
        visDone = true
      }
      if (arch?.exists && arch.content) {
        setArchitectureContent(arch.content)
        archDone = true
      }
      if (teamDoneNow) {
        setAgents(existingAgents)
      }

      setDone({
        descriptionDone: descDone,
        visionDone: visDone,
        architectureDone: archDone,
        teamDone: teamDoneNow,
      })

      // Jump to the first incomplete step
      if (!descDone) setCurrentStep(1)
      else if (!visDone) setCurrentStep(2)
      else if (!archDone) setCurrentStep(3)
      else if (!teamDoneNow) setCurrentStep(4)
      else setCurrentStep(5)
    })
  }, [])

  const onDocsEvent = useCallback((event: DocsSseEvent) => {
    if (event.type === 'vision_align_completed') {
      setAligningVision(false)
      api.getVision()
        .then(v => { if (v.content) setVisionContent(v.content) })
        .catch(() => {})
    }
    if (event.type === 'architecture_align_completed') {
      setAligningArchitecture(false)
      api.getArchitecture()
        .then(a => { if (a.content) setArchitectureContent(a.content) })
        .catch(() => {})
    }
    if (event.type === 'team_recruit_completed') {
      setRecruitingTeam(false)
      api.getProjectAgents()
        .then(a => {
          setAgents(a)
          setDone(prev => ({ ...prev, teamDone: true }))
        })
        .catch(() => {})
    }
  }, [])

  useSSE(() => {}, undefined, undefined, undefined, onDocsEvent)

  // Step 1 → Step 2: save config, auto-kick vision if missing
  const handleSaveDescription = async () => {
    if (!description.trim()) return
    setSaveError(null)
    setSaving(true)
    try {
      await api.updateConfig({ name: projectName.trim() || undefined, description: description.trim() })
      setDone(prev => ({ ...prev, descriptionDone: true }))
      if (!visionContent) {
        setAligningVision(true)
        api.runVisionAlign().catch(() => setAligningVision(false))
      }
      setCurrentStep(2)
    } catch (e) {
      setSaveError(e instanceof Error ? e.message : 'Failed to save')
    } finally {
      setSaving(false)
    }
  }

  // Step 2 → Step 3: save vision, auto-kick architecture if missing
  const handleSaveVision = async () => {
    setSavingVision(true)
    try {
      if (visionContent.trim()) await api.putVision(visionContent)
      setDone(prev => ({ ...prev, visionDone: true }))
      if (!architectureContent) {
        setAligningArchitecture(true)
        api.runArchitectureAlign().catch(() => setAligningArchitecture(false))
      }
      setCurrentStep(3)
    } catch {
      setDone(prev => ({ ...prev, visionDone: true }))
      setCurrentStep(3)
    } finally {
      setSavingVision(false)
    }
  }

  // Step 3 → Step 4: save architecture, auto-kick team recruit if no agents yet
  const handleSaveArchitecture = async () => {
    setSavingArchitecture(true)
    try {
      if (architectureContent.trim()) await api.putArchitecture(architectureContent)
      setDone(prev => ({ ...prev, architectureDone: true }))
      if (agents.length === 0) {
        setRecruitingTeam(true)
        api.runTeamRecruit().catch(() => setRecruitingTeam(false))
      }
      setCurrentStep(4)
    } catch {
      setDone(prev => ({ ...prev, architectureDone: true }))
      setCurrentStep(4)
    } finally {
      setSavingArchitecture(false)
    }
  }

  // Step 4 → Step 5: mark team done and advance
  const handleContinueFromTeam = () => {
    setDone(prev => ({ ...prev, teamDone: true }))
    setCurrentStep(5)
  }

  const allDone = done.descriptionDone && done.visionDone && done.architectureDone && done.teamDone

  const steps = [
    { n: 1, label: 'Project Info', done: done.descriptionDone },
    { n: 2, label: 'Vision', done: done.visionDone },
    { n: 3, label: 'Architecture', done: done.architectureDone },
    { n: 4, label: 'Team', done: done.teamDone },
    { n: 5, label: 'Get Started', done: allDone },
  ]

  return (
    <div className="max-w-2xl mx-auto p-6">
      <div className="mb-8">
        <h2 className="text-xl font-semibold">Set up your project</h2>
        <p className="text-sm text-muted-foreground mt-1">
          Let's get the basics in place so agents have context to work with.
        </p>
      </div>

      {/* Step indicator */}
      <div className="flex items-center gap-2 mb-8 flex-wrap">
        {steps.map((step, i) => (
          <div key={step.n} className="flex items-center gap-2">
            <button
              onClick={() => setCurrentStep(step.n)}
              className={`flex items-center gap-1.5 text-xs px-2.5 py-1 rounded-full transition-colors ${
                currentStep === step.n
                  ? 'bg-primary/15 text-primary border border-primary/30'
                  : step.done
                  ? 'text-green-400/80 border border-transparent'
                  : 'text-muted-foreground border border-transparent'
              }`}
            >
              {step.done ? (
                <CheckCircle className="w-3.5 h-3.5" />
              ) : (
                <Circle className="w-3.5 h-3.5" />
              )}
              {step.label}
            </button>
            {i < steps.length - 1 && (
              <div className="w-4 h-px bg-border/50 shrink-0" />
            )}
          </div>
        ))}
      </div>

      {/* Step 1: Project Info */}
      {currentStep === 1 && (
        <div className="space-y-4">
          <div className="flex items-center gap-2 mb-4">
            <Target className="w-5 h-5 text-muted-foreground" />
            <h3 className="text-base font-medium">Project Info</h3>
          </div>
          <div>
            <label className="block text-xs text-muted-foreground mb-1">Project name</label>
            <input
              type="text"
              value={projectName}
              onChange={e => setProjectName(e.target.value)}
              placeholder="my-project"
              className="w-full text-sm px-3 py-2 bg-background border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-ring font-mono"
            />
          </div>
          <div>
            <label className="block text-xs text-muted-foreground mb-1">
              Description
              <span className="text-muted-foreground/50 ml-1">— one sentence about what this project does</span>
            </label>
            <textarea
              value={description}
              onChange={e => setDescription(e.target.value)}
              placeholder="e.g. A CLI tool that tracks software features through structured lifecycle phases."
              rows={3}
              className="w-full text-sm px-3 py-2 bg-background border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-ring resize-none"
              onKeyDown={e => { if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) handleSaveDescription() }}
            />
            <p className="text-xs text-muted-foreground/50 mt-1">⌘↵ to save</p>
          </div>
          {saveError && <p className="text-xs text-destructive">{saveError}</p>}
          <div className="flex items-center gap-3 pt-1">
            <button
              onClick={handleSaveDescription}
              disabled={!description.trim() || saving}
              className="text-sm px-4 py-1.5 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-40"
            >
              {saving ? 'Saving…' : 'Save & Continue'}
            </button>
          </div>
        </div>
      )}

      {/* Step 2: Vision */}
      {currentStep === 2 && (
        <div className="space-y-4">
          <div className="flex items-center gap-2 mb-4">
            <Target className="w-5 h-5 text-muted-foreground" />
            <h3 className="text-base font-medium">Vision</h3>
          </div>

          {aligningVision ? (
            <div className="flex items-center gap-2 py-6 text-sm text-muted-foreground">
              <Loader2 className="w-4 h-4 animate-spin shrink-0" />
              Generating vision from your description…
            </div>
          ) : (
            <>
              <p className="text-xs text-muted-foreground">
                Edit the generated vision or write your own.{' '}
                <code className="text-primary">VISION.md</code> tells agents what you're building and why.
              </p>
              <textarea
                value={visionContent}
                onChange={e => setVisionContent(e.target.value)}
                placeholder="Describe your project's vision, goals, and what success looks like…"
                rows={12}
                className="w-full text-sm px-3 py-2 bg-background border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-ring resize-none font-mono"
              />
            </>
          )}

          <div className="flex items-center gap-3 pt-1">
            {!aligningVision && (
              <button
                onClick={handleSaveVision}
                disabled={savingVision}
                className="text-sm px-4 py-1.5 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-40"
              >
                {savingVision ? 'Saving…' : visionContent.trim() ? 'Save & Continue' : 'Continue'}
              </button>
            )}
          </div>
        </div>
      )}

      {/* Step 3: Architecture */}
      {currentStep === 3 && (
        <div className="space-y-4">
          <div className="flex items-center gap-2 mb-4">
            <GitBranch className="w-5 h-5 text-muted-foreground" />
            <h3 className="text-base font-medium">Architecture</h3>
          </div>

          {aligningArchitecture ? (
            <div className="flex items-center gap-2 py-6 text-sm text-muted-foreground">
              <Loader2 className="w-4 h-4 animate-spin shrink-0" />
              Generating architecture from your project…
            </div>
          ) : (
            <>
              <p className="text-xs text-muted-foreground">
                Edit the generated architecture or write your own.{' '}
                <code className="text-primary">ARCHITECTURE.md</code> maps your tech stack and key components.
              </p>
              <textarea
                value={architectureContent}
                onChange={e => setArchitectureContent(e.target.value)}
                placeholder="Describe your architecture, tech stack, key components, and design decisions…"
                rows={12}
                className="w-full text-sm px-3 py-2 bg-background border border-border rounded-lg focus:outline-none focus:ring-1 focus:ring-ring resize-none font-mono"
              />
            </>
          )}

          <div className="flex items-center gap-3 pt-1">
            {!aligningArchitecture && (
              <button
                onClick={handleSaveArchitecture}
                disabled={savingArchitecture}
                className="text-sm px-4 py-1.5 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-40"
              >
                {savingArchitecture ? 'Saving…' : architectureContent.trim() ? 'Save & Continue' : 'Continue'}
              </button>
            )}
          </div>
        </div>
      )}

      {/* Step 4: Team */}
      {currentStep === 4 && (
        <div className="space-y-4">
          <div className="flex items-center gap-2 mb-4">
            <Users className="w-5 h-5 text-muted-foreground" />
            <h3 className="text-base font-medium">Team</h3>
          </div>

          {recruitingTeam ? (
            <div className="flex items-center gap-2 py-6 text-sm text-muted-foreground">
              <Loader2 className="w-4 h-4 animate-spin shrink-0" />
              Recruiting thought partners for your project…
            </div>
          ) : agents.length > 0 ? (
            <>
              <p className="text-xs text-muted-foreground">
                These agents are now available as thought partners in your AI coding tools.
              </p>
              <div className="space-y-2">
                {agents.map(agent => (
                  <div
                    key={agent.name}
                    className="flex items-start gap-3 px-3 py-2.5 rounded-lg border border-border bg-muted/30"
                  >
                    <div className="w-7 h-7 rounded-full bg-primary/15 flex items-center justify-center shrink-0 mt-0.5">
                      <span className="text-xs font-semibold text-primary">
                        {agent.name.charAt(0).toUpperCase()}
                      </span>
                    </div>
                    <div className="min-w-0">
                      <p className="text-sm font-medium">{agent.name}</p>
                      {agent.description && (
                        <p className="text-xs text-muted-foreground mt-0.5">{agent.description}</p>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </>
          ) : (
            <p className="text-xs text-muted-foreground py-4">
              No agents recruited yet. You can always run{' '}
              <code className="text-primary">/sdlc-recruit</code> later to add thought partners.
            </p>
          )}

          {!recruitingTeam && (
            <div className="flex items-center gap-3 pt-1">
              <button
                onClick={handleContinueFromTeam}
                className="text-sm px-4 py-1.5 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
              >
                {agents.length > 0 ? 'Continue' : 'Continue'}
              </button>
            </div>
          )}
        </div>
      )}

      {/* Step 5: Get Started */}
      {currentStep === 5 && (
        <div className="space-y-6">
          <div className="flex items-center gap-2 mb-4">
            <Rocket className="w-5 h-5 text-muted-foreground" />
            <h3 className="text-base font-medium">You're ready to build</h3>
          </div>

          {allDone ? (
            <div className="flex items-center gap-2 text-sm text-green-400">
              <CheckCircle className="w-4 h-4" />
              All setup steps complete
            </div>
          ) : (
            <div className="text-sm text-muted-foreground">
              You skipped some steps — that's fine. You can always add them later from the Vision and Architecture pages.
            </div>
          )}

          <div className="space-y-3">
            <p className="text-xs text-muted-foreground">Start exploring ideas in the ponder workspace:</p>
            <CommandBlock cmd="/sdlc-ponder" />
          </div>

          <div className="space-y-3">
            <p className="text-xs text-muted-foreground">Or jump straight to planning:</p>
            <CommandBlock cmd="/sdlc-plan" />
          </div>

          <div className="pt-2">
            <Link
              to="/ponder"
              className="inline-flex items-center gap-1.5 text-sm px-4 py-1.5 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
            >
              Open Ponder Workspace →
            </Link>
          </div>
        </div>
      )}
    </div>
  )
}
