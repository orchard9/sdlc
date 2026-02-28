import { useState } from 'react'
import { X } from 'lucide-react'
import type { PonderTeamMember } from '@/lib/types'

// ---------------------------------------------------------------------------
// Deterministic avatar colour from name string
// ---------------------------------------------------------------------------

const AVATAR_COLOURS = [
  'bg-blue-600/80',
  'bg-violet-600/80',
  'bg-emerald-600/80',
  'bg-amber-600/80',
  'bg-rose-600/80',
  'bg-cyan-600/80',
  'bg-indigo-600/80',
  'bg-pink-600/80',
]

function avatarColour(name: string): string {
  let hash = 0
  for (let i = 0; i < name.length; i++) hash = name.charCodeAt(i) + ((hash << 5) - hash)
  return AVATAR_COLOURS[Math.abs(hash) % AVATAR_COLOURS.length]
}

function initials(name: string): string {
  return name
    .split(/\s+/)
    .slice(0, 2)
    .map(w => w[0]?.toUpperCase() ?? '')
    .join('')
}

// ---------------------------------------------------------------------------
// Member modal
// ---------------------------------------------------------------------------

function MemberModal({ member, onClose }: { member: PonderTeamMember; onClose: () => void }) {
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60"
      onClick={onClose}
    >
      <div
        className="bg-card border border-border rounded-xl p-6 max-w-md w-full shadow-2xl"
        onClick={e => e.stopPropagation()}
      >
        <div className="flex items-start justify-between mb-4">
          <div>
            <h3 className="text-base font-semibold">{member.name}</h3>
            <p className="text-sm text-primary/80 font-medium">{member.role}</p>
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded text-muted-foreground hover:text-foreground transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
        <div className="border-t border-border/50 pt-4">
          <p className="text-sm text-foreground/80 leading-relaxed">{member.context}</p>
        </div>
      </div>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Team row
// ---------------------------------------------------------------------------

interface Props {
  team: PonderTeamMember[]
}

export function TeamRow({ team }: Props) {
  const [activeModal, setActiveModal] = useState<PonderTeamMember | null>(null)

  return (
    <>
      <div className="flex items-center gap-2 flex-wrap">
        {team.map(member => (
          <button
            key={member.name}
            onClick={() => setActiveModal(member)}
            title={`${member.name} · ${member.role}`}
            className="flex flex-col items-center gap-1 group"
          >
            <div
              className={`
                w-9 h-9 rounded-full flex items-center justify-center text-xs font-bold text-white
                ${avatarColour(member.name)}
                ring-2 ring-transparent group-hover:ring-primary/40 transition-all
              `}
            >
              {initials(member.name)}
            </div>
            <span className="text-[10px] text-muted-foreground/60 group-hover:text-muted-foreground transition-colors max-w-12 truncate">
              {member.name.split(' ')[0]}
            </span>
          </button>
        ))}
      </div>

      {activeModal && (
        <MemberModal member={activeModal} onClose={() => setActiveModal(null)} />
      )}
    </>
  )
}
