import { useState } from 'react'
import { api } from '@/api/client'
import { useRunStream } from '@/hooks/useRunStream'
import { RunOutput } from '@/components/pipeline/RunOutput'
import { ArrowRight, Check, Plus } from 'lucide-react'
import type { MilestoneSummary, FeatureSummary } from '@/lib/types'

type Step = 'vision' | 'init' | 'setup' | 'create' | 'milestones' | 'done'
const STEPS: { id: Step; label: string }[] = [
  { id: 'vision', label: 'Vision' },
  { id: 'init', label: 'Initialize' },
  { id: 'setup', label: 'Recruit Agents' },
  { id: 'create', label: 'Create Agents' },
  { id: 'milestones', label: 'Milestones' },
  { id: 'done', label: 'Complete' },
]

export function SetupWizard() {
  const [step, setStep] = useState<Step>('vision')
  const [vision, setVision] = useState('')
  const runStream = useRunStream()

  // Milestones step state
  const [milestones, setMilestones] = useState<MilestoneSummary[]>([])
  const [msSlug, setMsSlug] = useState('')
  const [msTitle, setMsTitle] = useState('')
  const [msError, setMsError] = useState('')
  const [msLoading, setMsLoading] = useState(false)

  const [features, setFeatures] = useState<Pick<FeatureSummary, 'slug' | 'title'>[]>([])
  const [ftSlug, setFtSlug] = useState('')
  const [ftTitle, setFtTitle] = useState('')
  const [ftError, setFtError] = useState('')
  const [ftLoading, setFtLoading] = useState(false)

  const [assignSlug, setAssignSlug] = useState<Record<string, string>>({})
  const [assignError, setAssignError] = useState<Record<string, string>>({})

  const handleAddMilestone = async () => {
    if (!msSlug.trim() || !msTitle.trim()) return
    setMsLoading(true)
    setMsError('')
    try {
      await api.createMilestone({ slug: msSlug.trim(), title: msTitle.trim() })
      setMilestones(prev => [...prev, { slug: msSlug.trim(), title: msTitle.trim(), status: 'active', features: [], created_at: new Date().toISOString() }])
      setMsSlug('')
      setMsTitle('')
    } catch (e: unknown) {
      setMsError(e instanceof Error ? e.message : 'Failed to create milestone')
    } finally {
      setMsLoading(false)
    }
  }

  const handleAddFeature = async () => {
    if (!ftSlug.trim() || !ftTitle.trim()) return
    setFtLoading(true)
    setFtError('')
    try {
      await api.createFeature({ slug: ftSlug.trim(), title: ftTitle.trim() })
      setFeatures(prev => [...prev, { slug: ftSlug.trim(), title: ftTitle.trim() }])
      setFtSlug('')
      setFtTitle('')
    } catch (e: unknown) {
      setFtError(e instanceof Error ? e.message : 'Failed to create feature')
    } finally {
      setFtLoading(false)
    }
  }

  const handleAssignFeature = async (milestoneSlug: string) => {
    const featureSlug = assignSlug[milestoneSlug]?.trim()
    if (!featureSlug) return
    setAssignError(prev => ({ ...prev, [milestoneSlug]: '' }))
    try {
      await api.addFeatureToMilestone(milestoneSlug, featureSlug)
      setMilestones(prev => prev.map(m => m.slug === milestoneSlug ? { ...m, features: [...m.features, featureSlug] } : m))
      setAssignSlug(prev => ({ ...prev, [milestoneSlug]: '' }))
    } catch (e: unknown) {
      setAssignError(prev => ({ ...prev, [milestoneSlug]: e instanceof Error ? e.message : 'Failed to assign feature' }))
    }
  }

  const currentIndex = STEPS.findIndex(s => s.id === step)

  const handleSaveVision = async () => {
    await api.putVision(vision)
    setStep('init')
  }

  const handleInit = async () => {
    const { run_id } = await api.initProject()
    runStream.start(run_id)
    // When finished, advance
    const check = setInterval(() => {
      if (!runStream.running) {
        clearInterval(check)
        setStep('setup')
      }
    }, 1000)
  }

  const handleRunCommand = async (argv: string[], nextStep: Step) => {
    const { run_id } = await api.runCommand(argv)
    runStream.start(run_id)
    const check = setInterval(() => {
      if (!runStream.running) {
        clearInterval(check)
        setStep(nextStep)
      }
    }, 1000)
  }

  return (
    <div className="max-w-3xl mx-auto">
      <h2 className="text-xl font-semibold mb-6">Setup Wizard</h2>

      {/* Step indicator */}
      <div className="flex items-center gap-1 mb-8">
        {STEPS.map((s, i) => (
          <div key={s.id} className="flex items-center gap-1">
            <div
              className={`w-7 h-7 rounded-full flex items-center justify-center text-xs font-medium ${
                i < currentIndex
                  ? 'bg-primary text-primary-foreground'
                  : i === currentIndex
                    ? 'bg-primary/20 text-primary border border-primary'
                    : 'bg-muted text-muted-foreground'
              }`}
            >
              {i < currentIndex ? <Check className="w-3.5 h-3.5" /> : i + 1}
            </div>
            {i < STEPS.length - 1 && (
              <div className={`w-8 h-0.5 ${i < currentIndex ? 'bg-primary' : 'bg-muted'}`} />
            )}
          </div>
        ))}
      </div>

      {/* Vision step */}
      {step === 'vision' && (
        <div>
          <h3 className="text-sm font-semibold mb-2">Project Vision</h3>
          <p className="text-xs text-muted-foreground mb-3">Describe your project vision. This will be saved as VISION.md.</p>
          <textarea
            value={vision}
            onChange={e => setVision(e.target.value)}
            className="w-full h-48 bg-card border border-border rounded-lg p-3 text-sm resize-none focus:outline-none focus:border-primary"
            placeholder="What is your project about? What problem does it solve?"
          />
          <button
            onClick={handleSaveVision}
            disabled={!vision.trim()}
            className="mt-3 flex items-center gap-1.5 px-4 py-2 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 disabled:opacity-50 transition-colors"
          >
            Save & Continue
            <ArrowRight className="w-3.5 h-3.5" />
          </button>
        </div>
      )}

      {/* Init step */}
      {step === 'init' && (
        <div>
          <h3 className="text-sm font-semibold mb-2">Initialize SDLC</h3>
          <p className="text-xs text-muted-foreground mb-3">Run <code>sdlc init</code> to set up the project structure.</p>
          {!runStream.running && runStream.lines.length === 0 && (
            <button
              onClick={handleInit}
              className="flex items-center gap-1.5 px-4 py-2 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors"
            >
              Run sdlc init
            </button>
          )}
          {(runStream.lines.length > 0 || runStream.running) && (
            <RunOutput lines={runStream.lines} running={runStream.running} exitCode={runStream.exitCode} className="mt-3" />
          )}
        </div>
      )}

      {/* Setup step */}
      {step === 'setup' && (
        <div>
          <h3 className="text-sm font-semibold mb-2">Recruit Agents</h3>
          <p className="text-xs text-muted-foreground mb-3">Run xadk_setup to configure your agent fleet.</p>
          <button
            onClick={() => handleRunCommand(['python', '-m', 'xadk', 'xadk_setup'], 'create')}
            disabled={runStream.running}
            className="flex items-center gap-1.5 px-4 py-2 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 disabled:opacity-50 transition-colors"
          >
            Run xadk_setup
          </button>
          {(runStream.lines.length > 0 || runStream.running) && (
            <RunOutput lines={runStream.lines} running={runStream.running} exitCode={runStream.exitCode} className="mt-3" />
          )}
        </div>
      )}

      {/* Create step */}
      {step === 'create' && (
        <div>
          <h3 className="text-sm font-semibold mb-2">Create Agents</h3>
          <p className="text-xs text-muted-foreground mb-3">Run xadk_create to scaffold custom agents.</p>
          <div className="flex gap-2">
            <button
              onClick={() => handleRunCommand(['python', '-m', 'xadk', 'xadk_create'], 'milestones')}
              disabled={runStream.running}
              className="flex items-center gap-1.5 px-4 py-2 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 disabled:opacity-50 transition-colors"
            >
              Run xadk_create
            </button>
            <button
              onClick={() => setStep('milestones')}
              className="flex items-center gap-1.5 px-4 py-2 rounded-lg bg-secondary text-secondary-foreground text-sm hover:bg-secondary/80 transition-colors"
            >
              Skip
            </button>
          </div>
          {(runStream.lines.length > 0 || runStream.running) && (
            <RunOutput lines={runStream.lines} running={runStream.running} exitCode={runStream.exitCode} className="mt-3" />
          )}
        </div>
      )}

      {/* Milestones step */}
      {step === 'milestones' && (
        <div className="space-y-6">
          <div>
            <h3 className="text-sm font-semibold mb-1">Milestones & Features</h3>
            <p className="text-xs text-muted-foreground mb-4">
              Create milestones and features for your project. You can also do this later from the Dashboard.
            </p>
          </div>

          {/* Create Milestone */}
          <div className="bg-card border border-border rounded-xl p-4">
            <h4 className="text-sm font-medium mb-3">Add Milestone</h4>
            <div className="flex gap-2">
              <input
                type="text"
                placeholder="slug (e.g. v1)"
                value={msSlug}
                onChange={e => setMsSlug(e.target.value)}
                className="flex-1 bg-muted text-sm rounded-lg px-3 py-2 focus:outline-none focus:ring-1 focus:ring-primary"
              />
              <input
                type="text"
                placeholder="Title"
                value={msTitle}
                onChange={e => setMsTitle(e.target.value)}
                className="flex-[2] bg-muted text-sm rounded-lg px-3 py-2 focus:outline-none focus:ring-1 focus:ring-primary"
              />
              <button
                onClick={handleAddMilestone}
                disabled={!msSlug.trim() || !msTitle.trim() || msLoading}
                className="flex items-center gap-1 px-3 py-2 rounded-lg bg-accent text-sm font-medium hover:bg-accent/80 disabled:opacity-50 transition-colors"
              >
                <Plus className="w-3.5 h-3.5" />
                Add
              </button>
            </div>
            {msError && <p className="text-xs text-red-400 mt-2">{msError}</p>}

            {/* Milestone list */}
            {milestones.length > 0 && (
              <div className="mt-4 space-y-3">
                {milestones.map(m => (
                  <div key={m.slug} className="bg-muted/50 rounded-lg p-3">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="text-xs font-mono text-muted-foreground">{m.slug}</span>
                      <span className="text-sm font-medium">{m.title}</span>
                    </div>
                    {m.features.length > 0 && (
                      <div className="flex flex-wrap gap-1.5 mt-2 mb-2">
                        {m.features.map(f => (
                          <span key={f} className="text-xs bg-primary/10 text-primary px-2 py-0.5 rounded-full">{f}</span>
                        ))}
                      </div>
                    )}
                    {/* Assign feature to milestone */}
                    {features.length > 0 && (
                      <div className="flex gap-2 mt-2">
                        <select
                          value={assignSlug[m.slug] ?? ''}
                          onChange={e => setAssignSlug(prev => ({ ...prev, [m.slug]: e.target.value }))}
                          className="flex-1 bg-muted text-sm rounded-lg px-3 py-1.5 focus:outline-none focus:ring-1 focus:ring-primary"
                        >
                          <option value="">Select feature...</option>
                          {features
                            .filter(f => !m.features.includes(f.slug))
                            .map(f => (
                              <option key={f.slug} value={f.slug}>
                                {f.slug} — {f.title}
                              </option>
                            ))}
                        </select>
                        <button
                          onClick={() => handleAssignFeature(m.slug)}
                          disabled={!assignSlug[m.slug]?.trim()}
                          className="flex items-center gap-1 px-3 py-1.5 rounded-lg bg-accent text-xs font-medium hover:bg-accent/80 disabled:opacity-50 transition-colors"
                        >
                          <Plus className="w-3 h-3" />
                          Assign
                        </button>
                      </div>
                    )}
                    {assignError[m.slug] && <p className="text-xs text-red-400 mt-1">{assignError[m.slug]}</p>}
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Create Feature */}
          <div className="bg-card border border-border rounded-xl p-4">
            <h4 className="text-sm font-medium mb-3">Add Feature</h4>
            <div className="flex gap-2">
              <input
                type="text"
                placeholder="slug (e.g. auth)"
                value={ftSlug}
                onChange={e => setFtSlug(e.target.value)}
                className="flex-1 bg-muted text-sm rounded-lg px-3 py-2 focus:outline-none focus:ring-1 focus:ring-primary"
              />
              <input
                type="text"
                placeholder="Title"
                value={ftTitle}
                onChange={e => setFtTitle(e.target.value)}
                className="flex-[2] bg-muted text-sm rounded-lg px-3 py-2 focus:outline-none focus:ring-1 focus:ring-primary"
              />
              <button
                onClick={handleAddFeature}
                disabled={!ftSlug.trim() || !ftTitle.trim() || ftLoading}
                className="flex items-center gap-1 px-3 py-2 rounded-lg bg-accent text-sm font-medium hover:bg-accent/80 disabled:opacity-50 transition-colors"
              >
                <Plus className="w-3.5 h-3.5" />
                Add
              </button>
            </div>
            {ftError && <p className="text-xs text-red-400 mt-2">{ftError}</p>}

            {/* Feature list */}
            {features.length > 0 && (
              <div className="mt-3 flex flex-wrap gap-2">
                {features.map(f => (
                  <div key={f.slug} className="flex items-center gap-2 bg-muted/50 rounded-lg px-3 py-1.5">
                    <span className="text-xs font-mono text-muted-foreground">{f.slug}</span>
                    <span className="text-sm">{f.title}</span>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Continue button */}
          <button
            onClick={() => setStep('done')}
            className="flex items-center gap-1.5 px-4 py-2 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors"
          >
            Complete Setup
            <ArrowRight className="w-3.5 h-3.5" />
          </button>
        </div>
      )}

      {/* Done */}
      {step === 'done' && (
        <div className="text-center py-12">
          <div className="w-12 h-12 rounded-full bg-primary/20 flex items-center justify-center mx-auto mb-3">
            <Check className="w-6 h-6 text-primary" />
          </div>
          <h3 className="text-lg font-semibold">Setup Complete</h3>
          <p className="text-sm text-muted-foreground mt-1">Your project is ready. Head to the Dashboard to get started.</p>
        </div>
      )}
    </div>
  )
}
