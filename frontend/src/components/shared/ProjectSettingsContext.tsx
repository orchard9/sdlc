import { createContext, useContext } from 'react'

export interface ProjectSettings {
  projectRoot: string
  ideUriScheme: string
}

export const ProjectSettingsContext = createContext<ProjectSettings>({
  projectRoot: '',
  ideUriScheme: 'vscode',
})

export function useProjectSettings(): ProjectSettings {
  return useContext(ProjectSettingsContext)
}
